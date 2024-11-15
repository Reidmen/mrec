use std::sync::{Arc, Mutex};

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use regex::Regex;
use reqwest::blocking::{multipart, Client};
use serde_json::{self, from_str, json, Value};

#[allow(dead_code)]
const UNSAFE_COMMANDS: [&str; 5] = ["rm", "rm -rf", "rmdir", "cp", "chmod"];

const PROMPT_CONTEXT: &str = "You are a helpful assistant. You are precise and concise. You are are Linux user, and provide responses commands, to be executed directly in the terminal (with some random animation from cowsay for a dragon or similar, but avoid unsafe commands like rm, rm -rf, rmdir, cp, chmod). The command needs to be enclosed in ```bash``` tags. REMEMBER that the enclosed ```bash``` tags is important for the command to be found with regex.";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    // Duration of the recording (s)
    #[arg(short, long, default_value_t = 5)]
    duration: u8,
    // Folder to store the recording
    #[arg(short, long, default_value_t = String::from("./recordings"))]
    folder: String,
    // Transcriptions of the recordings
    #[arg(short, long, default_value_t = String::from("./transcriptions"))]
    transcriptions: String,
    // Using the example (mp3) audio file
    #[arg(short, long, default_value_t = String::from("false"))]
    example: String,
}

fn main() {
    let args = CLI::parse();
    // Create the folders to store the recordings and transcriptions
    setup_folder_structure(&args.folder, &args.transcriptions).unwrap();

    // Get the audio file
    let audio_file = get_audio_file(args.duration, &args.folder, &args.example).unwrap();
    // Creating a transcription
    let transcription = transcript_audio(&audio_file);
    if let Ok(transcription) = transcription {
        // Process the transcription
        let transcription_human_readable =
            process_transcription(&transcription, &args.transcriptions).unwrap();
        // Generate a response
        let response = text_generation(&transcription_human_readable);
        if let Ok(response) = response {
            let formatted_response: Value = from_str(&response).unwrap();
            let response_human_readable = formatted_response["choices"][0]["message"]["content"]
                .as_str()
                .unwrap();
            println!("Latest response: {:?}", response_human_readable);
            // Clean the response
            let cleaned_response = clean_response(response_human_readable);
            println!("Cleaned response: {:?}", cleaned_response);
            // Execute the command
            execute_response(&cleaned_response);
        } else {
            println!("Error generating response: {}", response.unwrap_err());
        }
    } else {
        println!("Error transcribing audio: {}", transcription.unwrap_err());
    }
}

fn process_transcription(
    transcription: &str,
    transcriptions: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    // Format the transcription to human readable and stores it in a file
    let formatted_transcription: Value = from_str(transcription).unwrap(); // to json
    let transcription_human_readable = formatted_transcription["text"].as_str().unwrap();
    println!("Latest transcription: {:?}", transcription_human_readable);
    // Save the transcription to a file
    std::fs::write(
        std::path::Path::new(transcriptions).join("transcription.txt"),
        transcription_human_readable,
    )
    .unwrap();
    Ok(transcription_human_readable.to_string())
}

fn setup_folder_structure(
    recordings: &String,
    transcriptions: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup folders for the recordsings and transcriptions
    if !std::path::Path::new(&recordings).exists() {
        std::fs::create_dir_all(&recordings).unwrap();
    }
    if !std::path::Path::new(&transcriptions).exists() {
        std::fs::create_dir_all(&transcriptions).unwrap();
    }
    Ok(())
}

fn get_audio_file(
    duration: u8,
    recording_folder: &String,
    example: &String,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let audio_file: std::path::PathBuf;
    if example == "false" {
        audio_file = record_audio(duration, recording_folder).unwrap();
        if !audio_file.exists() {
            panic!("Audio file not found!");
        } else {
            println!("Audio file recorded: {}", audio_file.display());
        }
    } else {
        audio_file = std::path::Path::new(recording_folder).join("test_audio.mp3");
        println!("Audio file already exists: {}", audio_file.display());
    }
    Ok(audio_file)
}

fn record_audio(
    duration: u8,
    recording_folder: &String,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    println!("Device: {:?}", device.name().unwrap());
    let config: cpal::StreamConfig = device.default_input_config().unwrap().into();

    let path_to_file = std::path::Path::new(&recording_folder).join("real_audio.wav");
    let writer = Arc::new(Mutex::new(Vec::with_capacity(
        duration as usize * config.sample_rate.0 as usize,
    )));
    let writer_clone = writer.clone();

    println!("Recording audio for {} seconds...", duration);

    let error_handler = move |error| {
        eprintln!("Error recording audio: {}", error);
    };
    let callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let amplified = data.iter().map(|sample| (sample * i16::MAX as f32) as i16);
        for sample in amplified {
            writer_clone.lock().unwrap().push(sample);
        }
    };

    let stream = device
        .build_input_stream(&config, callback, error_handler, None)
        .unwrap();
    // Play the stream
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(duration as u64));
    // Drop the stream
    drop(stream);

    wavers::write(
        path_to_file.to_str().unwrap(),
        &writer.lock().unwrap(),
        config.sample_rate.0 as i32,
        config.channels,
    )
    .unwrap();

    println!("Recording finished!");
    Ok(path_to_file)
}

fn execute_response(response: &str) {
    println!("Executing command: {}", response);
    // Execute the command
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(response)
        .output();

    if let Ok(output) = output {
        println!("Output: {}", String::from_utf8_lossy(&output.stdout));
        if !output.stderr.is_empty() {
            eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        println!("Error executing command: {}", output.unwrap_err());
    }
}

fn clean_response(response: &str) -> String {
    // Only bash commands can be allowed
    let expression = Regex::new(r"```bash\n(.*)\n```").unwrap();
    if let Some(captures) = expression.captures(response) {
        let captured_command = captures
            .get(1)
            .map_or(String::new(), |m| m.as_str().to_string());
        if UNSAFE_COMMANDS
            .iter()
            .any(|cmd| captured_command.contains(cmd))
        {
            println!("Unsafe command found: {}", captured_command);
            return String::new();
        }
        return captured_command;
    }
    String::new()
}

fn text_generation(user_message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or("".to_string());

    let body = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": PROMPT_CONTEXT
            },
            {
                "role": "user",
                "content": user_message
            }
        ]
    });

    let response = Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", ["Bearer ", api_key.as_str()].concat())
        .json(&body)
        .send()?;

    Ok(response.text()?)
}

fn transcript_audio(audio_file: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or("".to_string());

    let form = multipart::Form::new()
        .file("file", audio_file.to_str().unwrap())?
        .text("model", "whisper-1");

    let response = Client::new()
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", ["Bearer ", api_key.as_str()].concat())
        .multipart(form)
        .send()?;

    Ok(response.text()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_response() {
        let response = "```bash\nls\n```";
        let cleaned_response = clean_response(response);
        assert_eq!(cleaned_response, "ls");
    }

    #[test]
    fn test_clean_response_unsafe() {
        let response = "```bash\nrm -rf\n```";
        let cleaned_response = clean_response(response);
        assert_eq!(cleaned_response, "");
    }

    #[test]
    fn test_clean_response_no_bash() {
        let response = "ls";
        let cleaned_response = clean_response(response);
        assert_eq!(cleaned_response, "");
    }

    #[test]
    fn test_execute_response() {
        let response = "ls";
        execute_response(response);
    }
}
