//! Python bindings to the native Rust implementation.
#[path = "bindings/binding_alignmentoptions.rs"]
mod binding_alignmentoptions;
pub use binding_alignmentoptions::AlignmentOptions;
#[path = "bindings/binding_assignment.rs"]
mod binding_assignment;
pub use binding_assignment::Assignment;
#[path = "bindings/binding_assignments.rs"]
mod binding_assignments;
pub use binding_assignments::Assignments;
#[path = "bindings/binding_audio.rs"]
mod binding_audio;
pub use binding_audio::Audio;
#[path = "bindings/binding_audiooptions.rs"]
mod binding_audiooptions;
pub use binding_audiooptions::AudioOptions;
#[path = "bindings/binding_batchextraction.rs"]
mod binding_batchextraction;
pub use binding_batchextraction::BatchExtraction;
#[path = "bindings/binding_batchoptions.rs"]
mod binding_batchoptions;
pub use binding_batchoptions::BatchOptions;
#[path = "bindings/binding_batchoutputs.rs"]
mod binding_batchoutputs;
pub use binding_batchoutputs::BatchOutputs;
#[path = "bindings/binding_dataset.rs"]
mod binding_dataset;
pub use binding_dataset::Dataset;
#[path = "bindings/binding_datasets.rs"]
mod binding_datasets;
pub use binding_datasets::Datasets;
#[path = "bindings/binding_decodedmodel.rs"]
mod binding_decodedmodel;
pub use binding_decodedmodel::DecodedModel;
#[path = "bindings/binding_dithersequence.rs"]
mod binding_dithersequence;
pub use binding_dithersequence::DitherSequence;
#[path = "bindings/binding_duration.rs"]
mod binding_duration;
pub use binding_duration::Duration;
#[path = "bindings/binding_durations.rs"]
mod binding_durations;
pub use binding_durations::Durations;
#[path = "bindings/binding_extractor.rs"]
mod binding_extractor;
pub use binding_extractor::Extractor;
#[path = "bindings/binding_featurefiles.rs"]
mod binding_featurefiles;
pub use binding_featurefiles::FeatureFiles;
#[path = "bindings/binding_featureoptions.rs"]
mod binding_featureoptions;
pub use binding_featureoptions::FeatureOptions;
#[path = "bindings/binding_features.rs"]
mod binding_features;
pub use binding_features::Features;
#[path = "bindings/binding_filelikelihoods.rs"]
mod binding_filelikelihoods;
pub use binding_filelikelihoods::FileLikelihoods;
#[path = "bindings/binding_gaussian.rs"]
mod binding_gaussian;
pub use binding_gaussian::Gaussian;
#[path = "bindings/binding_gaussians.rs"]
mod binding_gaussians;
pub use binding_gaussians::Gaussians;
#[path = "bindings/binding_indexentries.rs"]
mod binding_indexentries;
pub use binding_indexentries::IndexEntries;
#[path = "bindings/binding_indexentry.rs"]
mod binding_indexentry;
pub use binding_indexentry::IndexEntry;
#[path = "bindings/binding_initializationoptions.rs"]
mod binding_initializationoptions;
pub use binding_initializationoptions::InitializationOptions;
#[path = "bindings/binding_isolatedgroup.rs"]
mod binding_isolatedgroup;
pub use binding_isolatedgroup::IsolatedGroup;
#[path = "bindings/binding_isolatedgroups.rs"]
mod binding_isolatedgroups;
pub use binding_isolatedgroups::IsolatedGroups;
#[path = "bindings/binding_iterationreport.rs"]
mod binding_iterationreport;
pub use binding_iterationreport::IterationReport;
#[path = "bindings/binding_iterationreports.rs"]
mod binding_iterationreports;
pub use binding_iterationreports::IterationReports;
#[path = "bindings/binding_jumpgroup.rs"]
mod binding_jumpgroup;
pub use binding_jumpgroup::JumpGroup;
#[path = "bindings/binding_label.rs"]
mod binding_label;
pub use binding_label::Label;
#[path = "bindings/binding_labels.rs"]
mod binding_labels;
pub use binding_labels::Labels;
#[path = "bindings/binding_likelihoodrows.rs"]
mod binding_likelihoodrows;
pub use binding_likelihoodrows::LikelihoodRows;
#[path = "bindings/binding_model.rs"]
mod binding_model;
pub use binding_model::Model;
#[path = "bindings/binding_modeldefinition.rs"]
mod binding_modeldefinition;
pub use binding_modeldefinition::ModelDefinition;
#[path = "bindings/binding_observation.rs"]
mod binding_observation;
pub use binding_observation::Observation;
#[path = "bindings/binding_observationstream.rs"]
mod binding_observationstream;
pub use binding_observationstream::ObservationStream;
#[path = "bindings/binding_observationstreams.rs"]
mod binding_observationstreams;
pub use binding_observationstreams::ObservationStreams;
#[path = "bindings/binding_observations.rs"]
mod binding_observations;
pub use binding_observations::Observations;
#[path = "bindings/binding_phonemap.rs"]
mod binding_phonemap;
pub use binding_phonemap::PhoneMap;
#[path = "bindings/binding_phonemapoptions.rs"]
mod binding_phonemapoptions;
pub use binding_phonemapoptions::PhoneMapOptions;
#[path = "bindings/binding_segmentation.rs"]
mod binding_segmentation;
pub use binding_segmentation::Segmentation;
#[path = "bindings/binding_segmentationdocument.rs"]
mod binding_segmentationdocument;
pub use binding_segmentationdocument::SegmentationDocument;
#[path = "bindings/binding_segmentations.rs"]
mod binding_segmentations;
pub use binding_segmentations::Segmentations;
#[path = "bindings/binding_segmentedutterances.rs"]
mod binding_segmentedutterances;
pub use binding_segmentedutterances::SegmentedUtterances;
#[path = "bindings/binding_segmentedwave.rs"]
mod binding_segmentedwave;
pub use binding_segmentedwave::SegmentedWave;
#[path = "bindings/binding_sptkprograms.rs"]
mod binding_sptkprograms;
pub use binding_sptkprograms::SptkPrograms;
#[path = "bindings/binding_states.rs"]
mod binding_states;
pub use binding_states::States;
#[path = "bindings/binding_stream.rs"]
mod binding_stream;
pub use binding_stream::Stream;
#[path = "bindings/binding_streams.rs"]
mod binding_streams;
pub use binding_streams::Streams;
#[path = "bindings/binding_trainingoptions.rs"]
mod binding_trainingoptions;
pub use binding_trainingoptions::TrainingOptions;
#[path = "bindings/binding_trainingresult.rs"]
mod binding_trainingresult;
pub use binding_trainingresult::TrainingResult;
#[path = "bindings/binding_untiedmodel.rs"]
mod binding_untiedmodel;
pub use binding_untiedmodel::UntiedModel;
#[path = "bindings/binding_utterancemodelsource.rs"]
mod binding_utterancemodelsource;
pub use binding_utterancemodelsource::UtteranceModelSource;
#[path = "bindings/binding_utteranceoptions.rs"]
mod binding_utteranceoptions;
pub use binding_utteranceoptions::UtteranceOptions;
#[path = "bindings/binding_wave.rs"]
mod binding_wave;
pub use binding_wave::Wave;
#[path = "bindings/binding_functions_batch.rs"]
mod binding_functions_batch;
pub use binding_functions_batch::*;
#[path = "bindings/binding_functions_feature.rs"]
mod binding_functions_feature;
pub use binding_functions_feature::*;
#[path = "bindings/binding_functions_index.rs"]
mod binding_functions_index;
pub use binding_functions_index::*;
#[path = "bindings/binding_functions_label.rs"]
mod binding_functions_label;
pub use binding_functions_label::*;
#[path = "bindings/binding_functions_rawfloat.rs"]
mod binding_functions_rawfloat;
pub use binding_functions_rawfloat::*;
use pyo3::prelude::*;
pub use shiro_rs::*;
mod api;
mod callbacks;
fn consumed() -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err("owner has been consumed")
}
#[pymodule]
fn _shiro_rs(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<AlignmentOptions>()?;
    module.add_class::<DitherSequence>()?;
    module.add_class::<AudioOptions>()?;
    module.add_class::<Wave>()?;
    module.add_class::<Audio>()?;
    module.add_function(wrap_pyfunction!(binding_batch_feature_options, module)?)?;
    module.add_class::<BatchOptions>()?;
    module.add_class::<BatchOutputs>()?;
    module.add_class::<BatchExtraction>()?;
    module.add_class::<ObservationStream>()?;
    module.add_class::<ObservationStreams>()?;
    module.add_class::<Observation>()?;
    module.add_class::<JumpGroup>()?;
    module.add_class::<Segmentation>()?;
    module.add_class::<Observations>()?;
    module.add_class::<Segmentations>()?;
    module.add_class::<Dataset>()?;
    module.add_class::<PhoneMap>()?;
    module.add_class::<States>()?;
    module.add_class::<SegmentationDocument>()?;
    module.add_class::<ModelDefinition>()?;
    module.add_class::<FeatureOptions>()?;
    module.add_class::<Features>()?;
    module.add_class::<SptkPrograms>()?;
    module.add_class::<Extractor>()?;
    module.add_function(wrap_pyfunction!(binding_batch_extract_file, module)?)?;
    module.add_class::<IndexEntry>()?;
    module.add_class::<IndexEntries>()?;
    module.add_function(wrap_pyfunction!(binding_index_append_suffix, module)?)?;
    module.add_class::<InitializationOptions>()?;
    module.add_class::<IsolatedGroup>()?;
    module.add_class::<IsolatedGroups>()?;
    module.add_class::<Label>()?;
    module.add_class::<Labels>()?;
    module.add_function(wrap_pyfunction!(binding_label_output_path, module)?)?;
    module.add_class::<FeatureFiles>()?;
    module.add_class::<Datasets>()?;
    module.add_class::<Duration>()?;
    module.add_class::<Durations>()?;
    module.add_class::<Gaussian>()?;
    module.add_class::<Gaussians>()?;
    module.add_class::<Stream>()?;
    module.add_class::<Streams>()?;
    module.add_class::<Model>()?;
    module.add_class::<DecodedModel>()?;
    module.add_function(wrap_pyfunction!(binding_feature_frame_count, module)?)?;
    module.add_class::<PhoneMapOptions>()?;
    module.add_function(wrap_pyfunction!(binding_rawfloat_read, module)?)?;
    module.add_function(wrap_pyfunction!(binding_rawfloat_write, module)?)?;
    module.add_class::<TrainingOptions>()?;
    module.add_class::<FileLikelihoods>()?;
    module.add_class::<LikelihoodRows>()?;
    module.add_class::<IterationReport>()?;
    module.add_class::<IterationReports>()?;
    module.add_class::<TrainingResult>()?;
    module.add_class::<Assignment>()?;
    module.add_class::<Assignments>()?;
    module.add_class::<UntiedModel>()?;
    module.add_class::<UtteranceOptions>()?;
    module.add_class::<UtteranceModelSource>()?;
    module.add_class::<SegmentedUtterances>()?;
    module.add_class::<SegmentedWave>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
