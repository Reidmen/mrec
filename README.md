# [M]minimal [REC]ording application

This is a minimal recording application built with Rust, designed to interact with the OpenAI Whisper API.

It's meant to be lightweight and easy to use, with a focus on simplicity and functionality. This was part of the **Rusty Day** at [Vanellus](https://vanellus.tech).

# Requirements
In order to run this application, you need to have a valid OpenAI API key. Export it as an environment variable:
```bash
export OPENAI_API_KEY=sk-proj-...
```

Some audio dependencies are required to be installed on your system:
```bash
sudo apt install libasound-dev
```
and `pv` and `cowsay` to be installed:
```bash
sudo apt install pv cowsay
```

# Usage
To execute the example audio file, run:
```bash
cargo run -- --example true
```

To record an audio with the default microphone, run:
```bash
cargo run --example false --duration 5
```
which will record for 5 seconds, request a transcription to the OpenAI Whisper API, 
process it and send it to a text-generating AI and execute the response in the terminal.


## Example response
An example response using the example audio file:
```bash
Cleaned response: "cowsay -f dragon \"Choo Choo! All aboard the Linux train!\" | lolcat"
Executing command: cowsay -f dragon "Choo Choo! All aboard the Linux train!" | lolcat
Output:  ________________________________________
< Choo Choo! All aboard the Linux train! >
 ----------------------------------------
      \                    / \  //\
       \    |\___/|      /   \//  \\
            /0  0  \__  /    //  | \ \    
           /     /  \/_/    //   |  \  \  
           @_^_@'/   \/_   //    |   \   \ 
           //_^_/     \/_ //     |    \    \
        ( //) |        \///      |     \     \
      ( / /) _|_ /   )  //       |      \     _\
    ( // /) '/,_ _ _/  ( ; -.    |    _ _\.-~        .-~~~^-.
  (( / / )) ,-{        _      `-.|.-~-.           .~         `.
 (( // / ))  '/\      /                 ~-. _ .-~      .-~^-.  \
 (( /// ))      `.   {            }                   /      \  \
  (( / ))     .----~-.\        \-'                 .~         \  `. \^-.
             ///.----..>        \             _ -~             `.  ^-`  ^-_
               ///-._ _ _ _ _ _ _}^ - - - - ~                     ~-- ,.-~
                                                                  /.-~
```

# TODO:

- [x] Add a way to save the recording to a file
- [x] Introduce basic CLI functionality
- [x] Store the recording in a folder
- [x] Create a request compatible with the OpenAI Whisper API
- [x] Process the request and save the response to a file
- [x] Execute such a response (e.g. as a "safe" linux command)
- [x] Testing for unsafe commands
- [ ] Save files with unique labels 


### Use at your own risk
This is a toy project, use at your own risk.
