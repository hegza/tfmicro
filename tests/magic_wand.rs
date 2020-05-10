extern crate itertools;
use itertools::Itertools;

use tfmicro::{
    micro_error_reporter::MicroErrorReporter,
    micro_interpreter::MicroInterpreter, micro_op_resolver::MicroOpResolver,
    model::Model,
};

use log::info;

#[test]
fn magic_wand() {
    env_logger::init();
    info!("---- Starting tensorflow micro example: magic_wand");

    let model = include_bytes!("../examples/models/magic_wand.tflite");
    let ring =
        &include_bytes!("../examples/models/ring_micro_f9643d42_nohash_4.data")
            .chunks_exact(4)
            .map(|c| f32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect_vec();
    let slope = &include_bytes!(
        "../examples/models/slope_micro_f2e59fea_nohash_1.data"
    )
    .chunks_exact(4)
    .map(|c| f32::from_be_bytes([c[0], c[1], c[2], c[3]]))
    .collect_vec();

    // Instantiate the model from the file
    let model = Model::from_buffer(&model[..]).unwrap();

    const TENSOR_ARENA_SIZE: usize = 60 * 1024;
    let mut tensor_arena: [u8; TENSOR_ARENA_SIZE] = [0; TENSOR_ARENA_SIZE];

    let micro_op_resolver = MicroOpResolver::new_for_magic_wand();

    let error_reporter = MicroErrorReporter::new();
    let mut interpreter = MicroInterpreter::new(
        &model,
        micro_op_resolver,
        &mut tensor_arena,
        TENSOR_ARENA_SIZE,
        &error_reporter,
    );

    // Four indices:
    // WingScore
    // RingScore
    // SlopeScore
    // NegativeScore
    for &(input_data, output_max_idx) in [(ring, 1), (slope, 2)].iter().rev() {
        // Set input tensor
        let input = interpreter.input(0);
        assert_eq!(
            [1, 128, 3, 1],
            input.tensor_info().dims,
            "Invalid input tensor dimensions"
        );
        input.tensor_data_mut().clone_from_slice(input_data);

        // Run TensorFlow operation
        interpreter.invoke().unwrap();

        // Get output tensor
        let output = interpreter.output(0);
        assert_eq!(
            [1, 4],
            output.tensor_info().dims,
            "Invalid output tensor dimensions"
        );

        // Four indices:
        // WingScore
        // RingScore
        // SlopeScore
        // NegativeScore
        dbg!(output.tensor_data::<f32>());
        assert_eq!(
            output
                .tensor_data::<f32>()
                .into_iter()
                .position_max_by(|&a, &b| a.partial_cmp(b).unwrap())
                .unwrap(),
            output_max_idx
        );
    }

    info!("---- Done");
}
