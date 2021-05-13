use futures::{stream, Stream, StreamExt}; // 0.3.1
use magick_rust::{magick_wand_genesis, MagickWand};
use std::{
    env,
    path::{Path, PathBuf},
    sync::Once,
};
use tokio::fs::{self, DirEntry};
use tokio::io;
use tokio::time::Instant;

fn visit(path: impl Into<PathBuf>) -> impl Stream<Item = io::Result<DirEntry>> + Send + 'static {
    async fn one_level(path: PathBuf, to_visit: &mut Vec<PathBuf>) -> io::Result<Vec<DirEntry>> {
        let mut dir = fs::read_dir(path).await?;
        let mut files = Vec::new();

        while let Some(child) = dir.next_entry().await? {
            if child.metadata().await?.is_dir() {
                to_visit.push(child.path());
            } else {
                files.push(child)
            }
        }

        Ok(files)
    }

    stream::unfold(vec![path.into()], |mut to_visit| async {
        let path = to_visit.pop()?;
        let file_stream = match one_level(path, &mut to_visit).await {
            Ok(files) => stream::iter(files).map(Ok).left_stream(),
            Err(e) => stream::once(async { Err(e) }).right_stream(),
        };

        Some((file_stream, to_visit))
    })
    .flatten()
}

// Used to make sure MagickWand is initialized exactly once. Note that we
// do not bother shutting down, we simply exit when we're done.
static START: Once = Once::new();

async fn distort(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    let wand = MagickWand::new();
    let path = path.to_str().unwrap();
    let dest_path = format!("dest/{}", path);

    if Path::new(&dest_path).exists() {
        println!("exist!");
        return Ok(());
    }

    let start = Instant::now();

    wand.read_image(path).unwrap();

    let width = wand.get_image_width();
    let height = wand.get_image_height();
    let scale = 2.0;

    wand.implode(0.3, 0)?;
    wand.liquid_rescale_image(
        (width as f64 * 0.5).round() as usize,
        (height as f64 * 0.5).round() as usize,
        scale * 0.5,
        0.0,
    )?;
    wand.liquid_rescale_image(
        (width as f64 * 1.5).round() as usize,
        (height as f64 * 1.5).round() as usize,
        scale,
        0.0,
    )?;

    let image = wand.write_image_blob("png")?;
    fs::write(dest_path, image).await?;

    println!("took {:?}", start.elapsed());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let input_dir_name = &args[1];

    fs::create_dir_all(format!("dest/{}", input_dir_name)).await?;

    let paths = visit(input_dir_name);
    paths
        .for_each_concurrent(6, |entry| async {
            match entry {
                Ok(entry) => distort(entry.path()).await.unwrap(),
                Err(_) => {}
            }
        })
        .await;

    Ok(())
}
