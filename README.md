# [M]minimal [REC]ording application

This is a minimal recording application built with Rust, designed to interact with the OpenAI Whisper API.

It's meant to be lightweight and easy to use, with a focus on simplicity and functionality.

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


# TODO:

- [ ] Add a way to save the recording to a file
- [ ] Introduce basic CLI functionality
- [ ] Store the recording in a folder
- [ ] Create a request compatible with the OpenAI Whisper API
- [ ] Process the request and save the response to a file
- [ ] Execute such a response (e.g. as a "safe" linux command)
- [ ] Testing
