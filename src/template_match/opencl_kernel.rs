pub const OCL_KERNEL: &str = r#"
inline ulong sum_region(
    __global const ulong* integral,
    int x,
    int y,
    int width,
    int height,
    int image_width
) {
    int x2 = x + width - 1;
    int y2 = y + height - 1;

    ulong br = integral[y2 * image_width + x2];
    ulong bl = (x == 0) ? 0 : integral[y2 * image_width + (x - 1)];
    ulong tr = (y == 0) ? 0 : integral[(y - 1) * image_width + x2];
    ulong tl = (x == 0 || y == 0) ? 0 : integral[(y - 1) * image_width + (x - 1)];
    long sum = (long)br + (long)tl - (long)bl - (long)tr;


    return (ulong)sum;
}


inline ulong sum_region_squared(
    __global const ulong* integral_sq,
    int x,
    int y,
    int width,
    int height,
    int image_width
) {
    int x2 = x + width - 1;
    int y2 = y + height - 1;

    ulong br = integral_sq[y2 * image_width + x2];
    ulong bl = (x == 0) ? 0 : integral_sq[y2 * image_width + (x - 1)];
    ulong tr = (y == 0) ? 0 : integral_sq[(y - 1) * image_width + x2];
    ulong tl = (x == 0 || y == 0) ? 0 : integral_sq[(y - 1) * image_width + (x - 1)];
    long sum = (long)br + (long)tl - (long)bl - (long)tr;
    return (ulong)sum;
}


__kernel void segmented_match_integral(
    __global const ulong* integral,
    __global const ulong* integral_sq,
    __global const int4* segments,
    __global const int4* segments_slow,
    __global const float* segment_values,
    __global const float* segment_values_slow,
    const int num_segments,
    const int num_segments_slow,
    const float template_mean,
    const float template_mean_slow,
    const float template_sq_dev,
    const float template_sq_dev_slow,
    __global float* results,
    const int image_width,
    const int image_height,
    const int template_width,
    const int template_height,
    const float min_expected_corr,
    __global float* precision_buff
) {
    float precision = precision_buff[0];
    int idx = get_global_id(0);
    int result_width = image_width - template_width + 1;
    int result_height = image_height - template_height + 1;

    if (idx >= result_width * result_height) return;
    // results[idx] = 0.0;
    int x = idx % result_width;
    int y = idx / result_width;


    ulong patch_sum = sum_region(integral, x, y, template_width, template_height, image_width);
    ulong patch_sq_sum = sum_region_squared(integral_sq, x, y, template_width, template_height, image_width);
    


    float area = (float)(template_width * template_height);
    float mean_img = (float)(patch_sum) / area;
    float var_img = (float)(patch_sq_sum) - ((float)(patch_sum) * (float)(patch_sum)) / area;
    
    float nominator = 0.0f;
    for (int i = 0; i < num_segments; i++) {
        int4 seg = segments[i];
        float seg_val = segment_values[i];
        int seg_area = seg.z * seg.w;

        ulong region_sum = sum_region(integral, x + seg.x, y + seg.y, seg.z, seg.w, image_width);

        nominator += ((float)(region_sum) - mean_img * seg_area) * (seg_val - template_mean);
    }

    float denominator = sqrt(var_img * template_sq_dev);
    
    float corr = (denominator != 0.0f) ? (nominator / denominator) : -1.0f;



    if (corr < (min_expected_corr - 0.001)* precision) {
        results[idx] = corr;
        return;
    } else {
        float denominator_slow = sqrt(var_img * template_sq_dev_slow);
        float nominator_slow = 0.0f;
        for (int i = 0; i < num_segments_slow; i++) {
            int4 seg_slow = segments_slow[i];
            float seg_val_slow = segment_values_slow[i];
            int seg_area = seg_slow.z * seg_slow.w;

            ulong region_sum = sum_region(integral, x + seg_slow.x, y + seg_slow.y, seg_slow.z, seg_slow.w, image_width);

            nominator_slow += ((float)(region_sum) - mean_img * seg_area) * (seg_val_slow - template_mean);
        }
        float corr_slow = (denominator_slow != 0.0f) ? (nominator_slow / denominator_slow) : -1.0f;
        results[idx] = corr_slow;
    }    
}


__kernel void v2_segmented_match_integral_fast_pass(
    __global const ulong* integral,
    __global const ulong* integral_sq,
    __global const int4* segments,
    __global const float* segment_values,
    const int num_segments,
    const float template_mean,
    const float template_sq_dev,
    __global int2* results,
    const int image_width,
    const int image_height,
    const int template_width,
    const int template_height,
    const float min_expected_corr,
    const int remainder_segments_fast,
    const int segments_per_thread_fast,
    const int pixels_per_workgroup,
    const int workgroup_size,
    __local ulong* sum_template_region_buff,
    __local ulong* sum_sq_template_region_buff,
    __local float* thread_segment_sum_buff,
    __global int* valid_corr_count,
    __global float* precision_buff
) {
    int global_id = get_global_id(0);
    int local_id = get_local_id(0);
    int workgroup_id = get_group_id(0);
    int result_w = image_width - template_width;
    if (local_id == 3 && global_id == 2) {
        valid_corr_count[0] == 0;
    }


    // num_segments is also count of threads per pixel for fast img
    if (local_id * segments_per_thread_fast +  remainder_segments_fast >= num_segments * pixels_per_workgroup) return ; // this solves more segments per thread

    int pixel_pos = (workgroup_id * pixels_per_workgroup) + (local_id / num_segments) ;
    int image_x = pixel_pos % result_w;
    int image_y = pixel_pos / result_w;

    // first sum the region of template area for numerator calculations
    // we do it with first threads for each x,y position which workgroup processes
    // if there are 5 pixels processed, local_id 0-4 should process sum regions for each position, 5-9 for squared
    ulong patch_sum = 0;
    if (local_id < pixels_per_workgroup) {
        patch_sum = sum_region(integral, image_x, image_y, template_width, template_height, image_width);
        sum_template_region_buff[local_id] = patch_sum;
        
    }
    
    // there will never be less than 2 segments 
    // meaning pixels per workgroup is never greater than workgroup_size / 2 
    if (local_id >= pixels_per_workgroup && local_id < pixels_per_workgroup * 2) {
        ulong patch_sq_sum = sum_region_squared(integral_sq, image_x, image_y, template_width, template_height, image_width);
        sum_sq_template_region_buff[local_id % pixels_per_workgroup] = patch_sq_sum;
    }
    
    int result_width = image_width - template_width + 1;
    int result_height = image_height - template_height + 1;
    float area = (float)(template_width * template_height);

    // wait  for threads to complete writing sum_area
    barrier(CLK_LOCAL_MEM_FENCE);

    
    float mean_img = (float)(sum_template_region_buff[local_id / num_segments]) / area;


    // this is to cover if we have more than 1 segment per thread. This method 
    // with remainder allows us to keep all threads working
    int remainder_offset = 0;
    int remainder_addition = 0;
    if (remainder_segments_fast > 0) {
        if (local_id >= remainder_segments_fast) {
            remainder_offset = remainder_segments_fast;
        } else {
            remainder_offset = local_id;
            remainder_addition = 1; 
        }
    
    }

    
    
    // AUDIT - DOUBLE CHECK THIS LOGIC
    int thread_segment_start = (local_id * segments_per_thread_fast + remainder_offset ) % num_segments;
    int thread_segment_end = thread_segment_start +  segments_per_thread_fast + remainder_addition;

    float nominator = 0.0f;
    for (int i = thread_segment_start; i< thread_segment_end; i++) {
        
        int4 seg = segments[i];
        float seg_val = segment_values[i];
        int seg_area = seg.z* seg.w;
        ulong region_sum = sum_region(integral, image_x + seg.x, image_y + seg.y, seg.z, seg.w, image_width);
        

        nominator += ((float)(region_sum) - mean_img * seg_area) * (seg_val - template_mean);

    }
    
    thread_segment_sum_buff[local_id] = nominator;

    barrier(CLK_LOCAL_MEM_FENCE);


    
    if (local_id < pixels_per_workgroup) {
        float nominator_sum = 0.0f;
        int sum_start = local_id * num_segments;
        int sum_end = sum_start + (num_segments / segments_per_thread_fast ) - (remainder_segments_fast/segments_per_thread_fast);
        for (int i = sum_start; i< sum_end; i++) {
            nominator_sum = nominator_sum + thread_segment_sum_buff[i] ;
        }

        int pixel_pos_final = (workgroup_id * pixels_per_workgroup) + (local_id) ;
        int image_x = pixel_pos_final % result_w;
        int image_y = pixel_pos_final / result_w;

        float precision = precision_buff[0];
        ulong patch_sq_sum_extracted = sum_sq_template_region_buff[local_id];
        float var_img = (float)patch_sq_sum_extracted - ((float)patch_sum * (float)patch_sum)/ (float)area;
        float denominator = sqrt(var_img * (float)template_sq_dev);
        float corr = (denominator != 0.0f) ? (nominator_sum / denominator) : -1.0f;        
        
        if (corr >= (min_expected_corr - 0.01) * precision && corr < 2) {
        
            int index = atomic_add(valid_corr_count, 1);
            results[index] = (int2)(image_x, image_y);
            
        }
    } 
}



__kernel void v2_segmented_match_integral_slow_pass (
    __global const ulong* integral,
    __global const ulong* integral_sq,
    __global const int4* segments,
    __global const float* segment_values,
    const int num_segments,
    const float template_mean,
    const float template_sq_dev,
    __global int2* position_results,
    __global float* corr_results,
    const int image_width,
    const int image_height,
    const int template_width,
    const int template_height,
    const float min_expected_corr,
    const int remainder_segments_slow,
    const int segments_per_thread_slow,
    const int workgroup_size,
    __local ulong* sum_template_region_buff,
    __local ulong* sum_sq_template_region_buff,
    __local float* thread_segment_sum_buff,
    __global int* valid_corr_count_slow,
    __global int* valid_corr_count_fast,
    __global int2* fast_pass_results,
    __global float* precision_buff
) {
    
    int global_id = get_global_id(0);
    int local_id = get_local_id(0);
    int workgroup_id = get_group_id(0);

    
    
    

    int image_x = fast_pass_results[workgroup_id].x;
    int image_y = fast_pass_results[workgroup_id].y;

    int result_w = image_width - template_width;
    // num_segments is also count of threads per pixel for fast img
    if (local_id * segments_per_thread_slow +  remainder_segments_slow >= num_segments) return ; // this solves more segments per thread

    
    // first sum the region of template area for numerator calculations
    // we do it with first threads for each x,y position which workgroup processes
    // if there are 5 pixels processed, local_id 0-4 should process sum regions for each position, 5-9 for squared
    ulong patch_sum = 0;
    if (local_id == 0) {
        patch_sum = sum_region(integral, image_x, image_y, template_width, template_height, image_width);
        sum_template_region_buff[0] = patch_sum;
        
    }
    
    // there will never be less than 2 segments 
    // meaning pixels per workgroup is never greater than workgroup_size / 2 
    if (local_id == 1) {
        ulong patch_sq_sum = sum_region_squared(integral_sq, image_x, image_y, template_width, template_height, image_width);
        sum_sq_template_region_buff[0] = patch_sq_sum;
    }
    int result_width = image_width - template_width + 1;
    int result_height = image_height - template_height + 1;
    float area = (float)(template_width * template_height);
    // wait  for threads to complete writing sum_area
    barrier(CLK_LOCAL_MEM_FENCE);
    float mean_img = (float)(sum_template_region_buff[0]) / area;
    // this is to cover if we have more than 1 segment per thread. This method 
    

    // with remainder allows us to keep all threads working
    int remainder_offset = 0;
    int remainder_addition = 0;
    if (remainder_segments_slow > 0) {
        if (local_id >= remainder_segments_slow) {
            remainder_offset = remainder_segments_slow;
        } else {
            remainder_offset = local_id;
            remainder_addition = 1; 
        }
    
    }

    int thread_segment_start = (local_id * segments_per_thread_slow + remainder_offset ) % num_segments;
    int thread_segment_end = thread_segment_start +  segments_per_thread_slow + remainder_addition;


    float nominator = 0.0f;
    for (int i = thread_segment_start; i< thread_segment_end; i++) {
        
        int4 seg = segments[i];
        float seg_val = segment_values[i];
        int seg_area = seg.z* seg.w;
        ulong region_sum = sum_region(integral, image_x + seg.x, image_y + seg.y, seg.z, seg.w, image_width);
        

        nominator += ((float)(region_sum) - mean_img * seg_area) * (seg_val - template_mean);

    }
    
    thread_segment_sum_buff[local_id] = nominator;
    barrier(CLK_LOCAL_MEM_FENCE);
    if (local_id == 0) {
        float nominator_sum = 0.0f;
        int sum_start = 0;
        int sum_end = sum_start + (num_segments / segments_per_thread_slow ) - (remainder_segments_slow/segments_per_thread_slow);
        for (int i = sum_start; i< sum_end; i++) {
            nominator_sum = nominator_sum + thread_segment_sum_buff[i] ;
        }

        


        ulong patch_sq_sum_extracted = sum_sq_template_region_buff[0];
        float var_img = (float)patch_sq_sum_extracted - ((float)patch_sum * (float)patch_sum)/ (float)area;
        float denominator = sqrt(var_img * (float)template_sq_dev);
        float corr = (denominator != 0.0f) ? (nominator_sum / denominator) : -1.0f;        
        float precision = precision_buff[0];
        
        if (corr >= (min_expected_corr - 0.001) * precision  && corr < 2) {
            int index = atomic_add(valid_corr_count_slow, 1);
            position_results[index] = (int2)(image_x, image_y);
            corr_results[index] = corr;
        }
    } 
}


"#;
