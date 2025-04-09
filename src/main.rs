use csv::Writer;
use glob::glob;
use replay::{KonohaDifficulty, Replay};
use std::path::PathBuf;

mod replay;

fn main() {
    #[cfg(unix)]
    //TODO ask for path here later
    let root_folder = PathBuf::from(
        "/Nagi/SteamLibrary/steamapps/compatdata/3328480/pfx/drive_c/users/steamuser/AppData/Local/tgm4/savedata/",
    );
    #[cfg(windows)]
    let root_folder = std::env::var("APPDATA").expect("No APPDATA directory");

    let mut wtr = Writer::from_path("replays.csv").unwrap();
    wtr.write_record(&[
        "SteamID",
        "Date",
        "Mode",
        "Level",
        "Rule",
        "Time (seconds)",
        "Seed",
    ])
    .unwrap();

    for entry in glob(&format!(
        "{}/**/replay_data/**/*.bin",
        root_folder.display()
    ))
    .expect("Failed to read glob pattern")
    {
        match entry {
            Ok(path) => {
                let bytes = std::fs::read(&path).unwrap();
                match Replay::from_bytes(&bytes) {
                    Ok(r) => {
                        let mode = match r.mode {
                            replay::Mode::Marathon => "Marathon".into(),
                            replay::Mode::Master => "Master".into(),
                            replay::Mode::Normal => "Normal".into(),
                            replay::Mode::Konoha(diff) => {
                                if diff == KonohaDifficulty::Hard {
                                    "Konoha Hard".into()
                                } else {
                                    "Konoha Easy".into()
                                }
                            }
                            replay::Mode::Shiranui(tier, points) => {
                                format!("Shiranui (Tier: {tier}, Points: {points})")
                            }
                            replay::Mode::Asuka => "Asuka".into(),
                        };
                        wtr.serialize((
                            r.steamid,
                            r.timestamp,
                            mode,
                            r.level,
                            &r.rule,
                            r.time.as_secs(),
                            r.seed,
                        ))
                        .unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error on path {}: {e}", path.display())
                    }
                };
            }
            Err(e) => println!("{:?}", e),
        }
    }
    wtr.flush().unwrap();
}
