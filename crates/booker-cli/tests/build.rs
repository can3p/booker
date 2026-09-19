#[test]
fn a_new_project_builds_a_real_pdf() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("mia");
    booker_project::Project::create(&root, booker_project::Template::Novel, Some("Mia")).unwrap();

    let mut out = Vec::new();
    let code = booker_cli::run(
        booker_cli::Command::Build {
            project: root.clone(),
        },
        &mut out,
    )
    .unwrap();
    assert_eq!(code, booker_cli::OK, "{}", String::from_utf8_lossy(&out));

    let pdf = root.join("build").join("mia.pdf");
    let bytes = std::fs::read(&pdf).expect("the build wrote a PDF");
    assert!(bytes.starts_with(b"%PDF-"), "not a PDF: {:?}", &bytes[..8]);
    assert!(
        bytes.len() > 1000,
        "suspiciously small PDF: {} bytes",
        bytes.len()
    );
}
