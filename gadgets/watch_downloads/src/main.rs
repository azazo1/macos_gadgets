use std::thread;
use std::time::Duration;
use std::{collections::HashSet, fs};
use tracing::info;
use watch_downloads::{MDirEntry, cpcb_file};

fn main() {
    tracing_subscriber::fmt().init();

    let mut ignore_exts = HashSet::new();
    ignore_exts.insert("aria2");

    info!("watch_downloads launched.");
    let download_dir = dirs::download_dir().unwrap();
    let mut file_set: HashSet<MDirEntry> = HashSet::new();
    let mut first = true;
    loop {
        let mut cur_files: Vec<_> = fs::read_dir(&download_dir)
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        if !first {
            cur_files.sort_by_key(|x| x.metadata().unwrap().accessed().unwrap());
            for f in &cur_files {
                if !file_set.contains(&f.try_into().unwrap()) {
                    if let Some(s) = f.path().extension()
                        && ignore_exts.contains(s.to_string_lossy().to_string().as_str())
                    {
                        continue;
                    }
                    cpcb_file(f.path()).unwrap();
                    info!("copy: {}", f.path().to_string_lossy());
                    break;
                }
            }
        }
        first = false;
        file_set.clear();
        for f in cur_files {
            file_set.insert(f.try_into().unwrap());
        }
        thread::sleep(Duration::from_secs(1));
    }
}
