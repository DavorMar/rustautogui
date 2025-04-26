// run with cargo test --tests --release -- --nocapture

#[cfg(feature = "dev")]
pub mod tmpl_match_tests {
    use rustautogui::core::template_match::open_cl::OclVersion;
    use rustautogui::core::template_match::*;
    use rustautogui::data::{opencl::KernelStorage, opencl::*, PreparedData};
    use rustautogui::imgtools;


    #[test]
    fn testing_speeds() {
        let image_paths = vec![
            "tests/testing_images/algorithm_tests/Darts_main.png",
            "tests/testing_images/algorithm_tests/Darts_main.png",
            "tests/testing_images/algorithm_tests/Darts_main.png",
            "tests/testing_images/algorithm_tests/Socket_main.png",
            "tests/testing_images/algorithm_tests/Socket_main.png",
            "tests/testing_images/algorithm_tests/Socket_main.png",
            "tests/testing_images/algorithm_tests/Split_main.png",
            "tests/testing_images/algorithm_tests/Split_main.png",
            // "tests/testing_images/algorithm_tests/Split_main.png",
            "tests/testing_images/algorithm_tests/Split_main.png",
            "tests/testing_images/algorithm_tests/Split_main.png",
        ];
        let template_paths = vec![
            "tests/testing_images/algorithm_tests/Darts_template1.png",
            "tests/testing_images/algorithm_tests/Darts_template2.png",
            "tests/testing_images/algorithm_tests/Darts_template3.png",
            "tests/testing_images/algorithm_tests/Socket_template1.png",
            "tests/testing_images/algorithm_tests/Socket_template2.png",
            "tests/testing_images/algorithm_tests/Socket_template3.png",
            "tests/testing_images/algorithm_tests/Split_template1.png",
            "tests/testing_images/algorithm_tests/Split_template2.png",
            // "tests/testing_images/algorithm_tests/Split_template3.png",
            "tests/testing_images/algorithm_tests/Split_template4.png",
            "tests/testing_images/algorithm_tests/Split_template5.png",
        ];
        let target_positions: Vec<(i32, i32)> = vec![
            (206, 1),
            (60, 270),
            (454, 31),
            (197, 345),
            (81, 825),
            (359, 666),
            (969, 688),
            (713, 1389),
            (1273, 1667),
            (41, 53),
        ];
        #[cfg(not(feature = "lite"))]
        for ((img_val, tmpl_val), target_position) in
            image_paths.iter().zip(template_paths).zip(target_positions)
        {
            testing_run(*img_val, tmpl_val, target_position);
        }
    }

    fn segmented_run(
        template: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
        main_image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
        target_positions: (i32, i32),
        template_path: &str,
        custom: bool,
    ) {
        let mut threshold = None;
        let mut insert_str = String::new();
        if custom {
            threshold = Some(0.5);
            insert_str.push_str("custom");
        }
        let template_data = segmented_ncc::prepare_template_picture(&template, &false, threshold);
        let template_data = match template_data {
            PreparedData::Segmented(data) => data,
            _ => panic!(),
        };
        let mut dur = 0.0;
        let mut locations: Vec<(u32, u32, f32)> = Vec::new();
        if template_path == "testing_images2/Split_template3.png" {
        } else {
            let start = std::time::Instant::now();
            locations =
                segmented_ncc::fast_ncc_template_match(&main_image, 0.95, &template_data, &false);
            dur = start.elapsed().as_secs_f32();
        }
        let mut first_location = (0, 0, 0.0);

        if locations.len() > 0 {
            first_location = locations[0];
            println!(
                "Segmented {insert_str}: Location found at {}, {} and corr {}, time: {} ",
                first_location.0, first_location.1, locations[0].2, dur
            );
        }
        assert!(
            first_location.0 == target_positions.0 as u32
                && first_location.1 == target_positions.1 as u32
        );
    }

    #[cfg(feature = "opencl")]
    fn ocl_run(
        template: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
        main_image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
        target_positions: (i32, i32),
        custom: bool,
        ocl_v: OclVersion,
        image_width: u32,
        image_height: u32,
        template_width: u32,
        template_height: u32,
    ) {
        
        use rustautogui::RustAutoGui;

        
        let data = RustAutoGui::dev_setup_opencl(Some(1)).unwrap();

        
        let mut threshold = None;
        let mut insert_str = String::new();
        let mut v_string = String::new();
        if custom {
            threshold = Some(0.7);
            insert_str.push_str("custom");
        }
        match ocl_v {
            OclVersion::V1 => v_string.push('1'),
            OclVersion::V2 => v_string.push('2'),
        }

        // Create a context for that device

        let queue = data.ocl_queue;
        let program = data.ocl_program;
        
        //////////////////////////////////////////////////////////////////////// OPENCL V1

        let template_data = segmented_ncc::prepare_template_picture(&template, &false, threshold);
        let template_data = match template_data {
            PreparedData::Segmented(x) => x,
            _ => panic!(),
        };
        let gpu_pointers = GpuMemoryPointers::new(
            image_width,
            image_height,
            template_width,
            template_height,
            &queue,
            &template_data.template_segments_slow,
            &template_data.template_segments_fast,
        )
        .unwrap();
        let kernels = KernelStorage::new(
            &gpu_pointers,
            &program,
            &queue,
            image_width,
            image_height,
            template_width,
            template_height,
            template_data.template_segments_fast.len() as u32,
            template_data.template_segments_slow.len() as u32,
            template_data.segments_mean_fast,
            template_data.segments_mean_slow,
            template_data.segment_sum_squared_deviations_fast,
            template_data.segment_sum_squared_deviations_slow,
            template_data.expected_corr_fast,
            256,
        )
        .unwrap();
        let start = std::time::Instant::now();
        let locations = open_cl::gui_opencl_ncc_template_match(
            &queue,
            &program,
            256,
            &kernels,
            &gpu_pointers,
            0.95,
            &main_image,
            &template_data,
            ocl_v,
        )
        .unwrap();
        let mut first_location = (0, 0, 0.0);
        if locations.len() > 0 {
            first_location = locations[0];
            println!(
                "OCL V{v_string} {insert_str}: Location found at {:?}, time: {}",
                first_location,
                start.elapsed().as_secs_f32()
            );
        }
        assert!(
            first_location.0 == target_positions.0 as u32
                && first_location.1 == target_positions.1 as u32
        );
    }

    fn fft_run(
        template: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
        main_image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
        target_positions: (i32, i32),
        image_width: u32,
        image_height: u32,
    ) {
        // fft corr
        let template_data = fft_ncc::prepare_template_picture(&template, image_width, image_height);
        let start = std::time::Instant::now();
        let locations = fft_ncc::fft_ncc(&main_image, 0.90, &template_data);

        let mut first_location = (0, 0, 0.0);
        if locations.len() > 0 {
            first_location.0 = locations[0].0;
            first_location.1 = locations[0].1;
            println!(
                "FFT: Location found at {}, {} and corr {} , time: {}",
                first_location.0,
                first_location.1,
                locations[0].2,
                start.elapsed().as_secs_f32()
            );
        }
        assert!(
            first_location.0 == target_positions.0 as u32
                && first_location.1 == target_positions.1 as u32
        );
        println!("\n");
    }

    fn testing_run(image_path: &str, template_path: &str, target_positions: (i32, i32)) {
        let template: image::ImageBuffer<image::Luma<u8>, Vec<u8>> =
            imgtools::load_image_bw(template_path).unwrap();
        let main_image: image::ImageBuffer<image::Luma<u8>, Vec<u8>> =
            imgtools::load_image_bw(image_path).unwrap();
        let (image_width, image_height) = main_image.dimensions();
        let (template_width, template_height) = template.dimensions();

        println!("Image: {}, template:{}", image_path, template_path);

        segmented_run(
            &template,
            &main_image,
            target_positions,
            template_path,
            false,
        ); // default cpu
        segmented_run(
            &template,
            &main_image,
            target_positions,
            template_path,
            true,
        ); // cpu custom
        ocl_run(
            &template,
            &main_image,
            target_positions,
            false,
            OclVersion::V1,
            image_width,
            image_height,
            template_width,
            template_height,
        );
        ocl_run(
            &template,
            &main_image,
            target_positions,
            false,
            OclVersion::V2,
            image_width,
            image_height,
            template_width,
            template_height,
        );
        ocl_run(
            &template,
            &main_image,
            target_positions,
            true,
            OclVersion::V1,
            image_width,
            image_height,
            template_width,
            template_height,
        );
        ocl_run(
            &template,
            &main_image,
            target_positions,
            true,
            OclVersion::V2,
            image_width,
            image_height,
            template_width,
            template_height,
        );
        fft_run(
            &template,
            &main_image,
            target_positions,
            image_width,
            image_height,
        );
    }
}
