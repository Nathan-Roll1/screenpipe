// screenpipe — AI that knows everything you've seen, said, or heard
// https://screenpipe.com
use std::{path::Path, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let source = &args[1];
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let t = Instant::now();
    let mut model = if Path::new(source).is_dir() {
        audiopipe::Model::from_dir(Path::new(source), "parakeet")?
    } else {
        audiopipe::Model::from_pretrained(source)?
    };
    eprintln!("load_s={:.6}",t.elapsed().as_secs_f64());
    let mut clips = Vec::new();
    for row in manifest.as_array().ok_or("manifest must be an array")? {
        let path = row["path"].as_str().ok_or("missing path")?;
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        if spec.channels != 1 || spec.sample_rate != 16000 || spec.bits_per_sample != 16 {return Err("expected mono 16kHz PCM16".into())}
        let audio: Vec<f32> = reader.samples::<i16>().map(|x| x.map(|x|x as f32/32768.0)).collect::<Result<_,_>>()?;
        clips.push((row,audio));
    }
    if clips.is_empty() {return Err("empty corpus".into())}
    for _ in 0..2 {model.transcribe_with_sample_rate(&clips[0].1,16000,Default::default())?;}
    for pass in 0..args.get(3).map(|v|v.parse()).transpose()?.unwrap_or(2) {
        for (row,audio) in &clips {
            let t = Instant::now();
            let mut texts=Vec::new();
            for chunk in audio.chunks(16000*30) {
                let result=model.transcribe_with_sample_rate(chunk,16000,Default::default())?;
                if !result.text.trim().is_empty(){texts.push(result.text.trim().to_owned())}
            }
            println!("{}",serde_json::json!({"source":source,"id":row["id"],"language":row["language"],"reference":row["reference"],"hypothesis":texts.join(" "),"pass":pass,"latency_ms":t.elapsed().as_secs_f64()*1000.0,"audio_s":audio.len() as f64/16000.0}));
        }
    }
    Ok(())
}
