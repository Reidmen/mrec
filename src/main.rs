use clap::Parser;
use regex::Regex;
use reqwest::blocking::{multipart, Client};
use serde_json::{self, from_str, json, Value};
#[allow(dead_code)]
const SAFE_COMMANDS: [&str; 19] = [
    "ls",
    "cat",
    "echo",
    "pwd",
    "cd",
    "mkdir",
    "mv",
    "cp",
    "touch",
    "tar",
    "chgrp",
    "ln",
    "ln -s",
    "chmod",
    "cd",
    "git",
    "git diff",
    "git status",
    "find",
];

const PROMPT_CONTEXT: &str = "You are a helpful assistant. You are precise and concise. You are are Linux user, and provide as responses a cheerful and funny command that creates a nice animation with the cowsay command, to be executed directly in the terminal. The command needs to be enclosed in ```bash``` tags.";

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
}

fn main() {
    let args = CLI::parse();

    println!("Duration: {}", args.duration);
    // Create folder if is doesn't exist
    if !std::path::Path::new(&args.folder).exists() {
        std::fs::create_dir_all(&args.folder).unwrap();
    }
    // Get the audio file if it doesn't exits
    let audio_file = std::path::Path::new(&args.folder).join("test_audio.mp3");
    if !audio_file.exists() {
        // The logic here must be implemented and checked the file is stored in a folder
        println!("TEST Recording audio...");
    } else {
        println!("Audio file already exists: {}", audio_file.display());
    }
    println!("Folder: {}", args.folder);
    if !std::path::Path::new(&args.transcriptions).exists() {
        std::fs::create_dir_all(&args.transcriptions).unwrap();
    }
    println!("Transcriptions folder: {}", args.transcriptions);

    // Creating a transcription
    let transcription = transcript_audio(&audio_file);
    if let Ok(transcription) = transcription {
        let formatted_transcription: Value = from_str(&transcription).unwrap();
        let transcription_human_readable = formatted_transcription["text"].as_str().unwrap();
        println!("Latest transcription: {:?}", transcription_human_readable);
        // Save the transcription to a file
        std::fs::write(
            std::path::Path::new(&args.transcriptions).join("transcription.txt"),
            transcription_human_readable,
        )
        .unwrap();
        // Generate a response
        let response = text_generation(transcription_human_readable);
        if let Ok(response) = response {
            let formatted_response: Value = from_str(&response).unwrap();
            let response_human_readable = formatted_response["choices"][0]["message"]["content"]
                .as_str()
                .unwrap();
            println!("Latest response: {:?}", response_human_readable);
            // Clean the response
            let cleaned_response = clean_response(response_human_readable);
            // Execute the command
            execute_response(&cleaned_response);
        } else {
            println!("Error generating response: {}", response.unwrap_err());
        }
    } else {
        println!("Error transcribing audio: {}", transcription.unwrap_err());
    }
}

fn execute_response(response: &str) {
    println!("Executing command: {}", response);
    // Execute the command
    let output = std::process::Command::new("bash").arg(response).output();

    if let Ok(output) = output {
        println!("Output: {:?}", String::from_utf8(output.stdout).unwrap());
    } else {
        println!("Error executing command: {}", output.unwrap_err());
    }
}

fn clean_response(response: &str) -> String {
    // Only bash commands can be allowed
    let expression = Regex::new(r"```bash\n(.*)\n```").unwrap();
    if let Some(captures) = expression.captures(response) {
        return captures
            .get(1)
            .map_or(String::new(), |m| m.as_str().to_string());
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
