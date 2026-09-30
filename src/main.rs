use self_update::cargo_crate_version;
#[cfg(not(feature = "signatures"))]
use self_update::http::status;

const REPO_OWNER: &str = "Nisenogen";
const REPO_NAME: &str = "self_update_test";
const BIN_NAME: &str = "self_update_test";
static ARCHIVE_PATH: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let mut archive_path = String::from(REPO_OWNER);
    archive_path.push('/');
    archive_path.push_str(REPO_NAME);
    archive_path
});

fn main() {
    println!("Version {}", cargo_crate_version!());
    let mut user_input = String::new();

    'mainloop: loop {
        println!("Press 'c' to check for latest version, 'u' to update, or 'q' to quit");
        user_input.clear();
        std::io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read user input");

        match user_input.trim_end() {
            "q" => break 'mainloop,
            "c" => {
                let check_result = check();
                if let Err(error) = check_result {
                    eprintln!("Error checking for update: {error}");
                }
            }
            "u" => {
                let check_result = update();
                if let Err(error) = check_result {
                    eprintln!("Error performing self update: {error}");
                }
            }
            _ => (),
        }
    }
}

fn check() -> Result<(), Box<dyn std::error::Error>> {
    let update = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(cargo_crate_version!())
        .build()?;

    match update.is_update_available()? {
        Some(release) => println!("Update available: {}", release.version()),
        None => println!(
            "No update available, current version {} is latest",
            cargo_crate_version!()
        ),
    }

    Ok(())
}

fn update() -> Result<(), Box<dyn std::error::Error>> {
    let mut status_builder = self_update::backends::github::Update::configure();

    #[cfg(feature = "signatures")]
    status_builder
        .repo_owner(REPO_OWNER)
        .verifying_keys([*include_bytes!("github-public.key")]);
    #[cfg(not(feature = "signatures"))]
    status_builder.repo_owner(REPO_OWNER);

    let status = status_builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(cargo_crate_version!())
        .show_download_progress(true)
        /*
        .verify_archive(|archive: &std::path::Path| {
            let ok = std::process::Command::new("gh")
                .args(["attestation", "verify"])
                .arg(archive)
                .args(["--repo", ARCHIVE_PATH.as_str()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if ok {
                Ok(())
            } else {
                Err(self_update::Error::archive_verification_rejected(
                    "No build-provenance attestation for this artifact",
                ))
            }
        })
        */
        .build()?
        .update()?;

    println!("Update status: '{}'!", status.version());

    if let self_update::VersionStatus::Updated(_) = status
        && self_update::restart::restart().is_err()
    {
        println!("Restart attempted but failed!");
    }

    Ok(())
}
