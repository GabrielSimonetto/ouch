use std::{
    env, fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

use ouch::{cli::Command, commands::run, oof};
use rand::{rngs::SmallRng, RngCore, SeedableRng};

#[test]
/// Tests each format that supports multiple files with random input.
/// TODO: test the remaining formats.
fn test_each_format() {
    test_compressing_and_decompressing_archive("tar");
    test_compressing_and_decompressing_archive("tar.gz");
    test_compressing_and_decompressing_archive("tar.bz");
    test_compressing_and_decompressing_archive("tar.bz2");
    test_compressing_and_decompressing_archive("tar.xz");
    test_compressing_and_decompressing_archive("tar.lz");
    test_compressing_and_decompressing_archive("tar.lzma");
    test_compressing_and_decompressing_archive("tar.zst");
    test_compressing_and_decompressing_archive("zip");
    test_compressing_and_decompressing_archive("zip.gz");
    test_compressing_and_decompressing_archive("zip.bz");
    test_compressing_and_decompressing_archive("zip.bz2");
    test_compressing_and_decompressing_archive("zip.xz");
    test_compressing_and_decompressing_archive("zip.lz");
    test_compressing_and_decompressing_archive("zip.lzma");
    test_compressing_and_decompressing_archive("zip.zst");

    // Why not
    test_compressing_and_decompressing_archive(
        "tar.gz.bz.xz.zst.gz.bz.xz.zst.gz.bz.xz.zst.gz.bz.xz.zst.gz.bz.xz.zst.gz.bz.xz.zst.gz.bz.xz.zst",
    );
}

type FileContent = Vec<u8>;

/// Compress and decompresses random files to archive formats, checks if contents match
fn test_compressing_and_decompressing_archive(format: &str) {
    // System temporary directory depends on the platform, for linux it's /tmp
    let system_tmp = env::temp_dir();

    // Create a temporary testing folder that will be deleted on scope drop
    let testing_dir =
        tempfile::Builder::new().prefix("ouch-testing").tempdir_in(system_tmp).expect("Could not create testing_dir");
    let testing_dir_path = testing_dir.path();

    // Quantity of compressed files vary from 1 to 10
    let mut rng = SmallRng::from_entropy();
    let quantity_of_files = rng.next_u32() % 10 + 1;

    let contents_of_files: Vec<FileContent> =
        (0..quantity_of_files).map(|_| generate_random_file_content(&mut rng)).collect();

    // Create them
    let mut file_paths = create_files(&testing_dir_path, &contents_of_files);
    // Compress them
    let compressed_archive_path = compress_files(&testing_dir_path, &file_paths, &format);
    // Decompress them
    let mut extracted_paths = extract_files(&compressed_archive_path);

    // // DEBUG UTIL:
    // // Uncomment line below to freeze the code and see compressed and extracted files in
    // // the temporary directory before their auto-destruction.
    // std::thread::sleep(std::time::Duration::from_secs(1_000_000));

    file_paths.sort();
    extracted_paths.sort();

    assert_correct_paths(&file_paths, &extracted_paths, format);
    compare_file_contents(&extracted_paths, &contents_of_files, format);
}

/// Crate file contents from 1024 up to 8192 random bytes
fn generate_random_file_content(rng: &mut impl RngCore) -> FileContent {
    let quantity = 1024 + rng.next_u32() % (8192 - 1024);
    let mut vec = vec![0; quantity as usize];
    rng.fill_bytes(&mut vec);
    vec
}

/// Create files using the indexes as file names (eg. 0, 1, 2 and 3)
///
/// Return the path to each one.
fn create_files(at: &Path, contents: &[FileContent]) -> Vec<PathBuf> {
    contents
        .iter()
        .enumerate()
        .map(|(i, content)| {
            let path = at.join(i.to_string());
            let mut file = fs::File::create(&path).expect("Could not create dummy test file");
            file.write_all(content).expect("Could not write to dummy test file");
            path
        })
        .collect()
}

fn compress_files(at: &Path, paths_to_compress: &[PathBuf], format: &str) -> PathBuf {
    let archive_path = String::from("archive.") + format;
    let archive_path = at.join(archive_path);

    let command = Command::Compress { files: paths_to_compress.to_vec(), output_path: archive_path.to_path_buf() };
    run(command, &oof::Flags::default()).expect("Failed to compress test dummy files");

    archive_path
}

fn extract_files(archive_path: &Path) -> Vec<PathBuf> {
    // We will extract in the same folder as the archive
    // If the archive is at:
    //   /tmp/ouch-testing-tar.Rbq4DusBrtF8/archive.tar
    // Then the extraction_output_folder will be:
    //   /tmp/ouch-testing-tar.Rbq4DusBrtF8/extraction_results/
    let mut extraction_output_folder = archive_path.to_path_buf();
    // Remove the name of the extracted archive
    assert!(extraction_output_folder.pop());
    // Add the suffix "results"
    extraction_output_folder.push("extraction_results");

    let command = Command::Decompress {
        files: vec![archive_path.to_owned()],
        output_folder: Some(extraction_output_folder.clone()),
    };
    run(command, &oof::Flags::default()).expect("Failed to extract");

    fs::read_dir(extraction_output_folder).unwrap().map(Result::unwrap).map(|entry| entry.path()).collect()
}

fn assert_correct_paths(original: &[PathBuf], extracted: &[PathBuf], format: &str) {
    assert_eq!(
        original.len(),
        extracted.len(),
        "Number of compressed files does not match number of decompressed when testing archive format '{:?}'.",
        format
    );
    for (original, extracted) in original.iter().zip(extracted) {
        assert_eq!(original.file_name(), extracted.file_name(), "");
    }
}

fn compare_file_contents(extracted: &[PathBuf], contents: &[FileContent], format: &str) {
    extracted.iter().zip(contents).for_each(|(extracted_path, expected_content)| {
        let actual_content = fs::read(extracted_path).unwrap();

        assert_eq!(
            expected_content,
            actual_content.as_slice(),
            "Contents of file with path '{:?}' does not match after compression and decompression while testing archive format '{:?}.'",
            extracted_path.canonicalize().unwrap(),
            format
        );
    });
}
