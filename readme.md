# Playdate SDK Updater

A simple tool to help you keep your Playdate SDK up to date.

## Features

- Automatically checks for the latest SDK version.
- Downloads and installs updates seamlessly.
- Cross-platform support.

## Installation (download from github)

1. Download the latest `playdate-sdk-updater` from the release tab on github
2. (optional) Download the signing key 
    check the signature with `gpg --verify playdate-sdk-updater.sig playdater-sdk-updater`
3. 

## Installation (build it yourself)

1. Clone the repository:
    ```bash
    git clone https://github.com/yourusername/playdate-sdk-updater.git
    ```
2. Navigate to the project directory:
    ```bash
    cd playdate-sdk-updater
    ```
3. Build it yourself (with Rust)
    ```bash
    cargo build --release
    ```
4. Copy the file to your local binary folder
    ```bash
    cp ./target/release/playdate-sdk-updater ~/.local/bin/
    ```

Enjoy !

## Usage

Run the updater:
```bash
./playdate-sdk-updater
```

## License

This project is licensed under the [MIT-ish License](LICENSE).

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Disclaimer

This project is not affiliated with Panic Inc or the official Playdate SDK.
