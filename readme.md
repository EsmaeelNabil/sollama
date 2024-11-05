# Sollama

Sollama is a Rust-based tool for scraping web content, performing searches, and processing prompts using a Language
Model (LLM).

## Prerequisites

- Rust (latest stable version)
- [Ollama](https://ollama.com/) downloaded and running

```shell
  # this will pull the model and run it
 ollama run llama3.2
```
- Then `/bye` to exit the chat session.
- Then `ollama serve` to start the server.

## Install

```bash
# Install hto
cargo install sollama
```

## Building the Project

To build the project, follow these steps:

1. Clone the repository:
    ```sh
    git clone https://github.com/esmaeelnabil/sollama.git
    cd sollama
    ```

2. Build the project using Cargo:
    ```sh
    cargo build --release
    ```

## Running the Tool

To run the tool, use the following command:

```sh
cargo run --release -- <search_query> <llm_query> <search_results_count> <llm_model>
```

- `<search_query>`: The search query to perform.
- `<llm_query>`: The query or question to be included in the prompt.
- `<search_results_count>`: The number of search results to return from Google: `default = 3`.
- `<llm_model>`: The model to be used for processing the prompt: `default = "llama3.2:latest"`.

Example:

```sh
cargo run --release -- "rust programming" "based on the content provided what is : rust programming" 5 "gpt-3"
```

## Testing

To run the tests, use the following command:

```sh
cargo test
```

### Future Work
- Using better cli args parsing library.
- Implementing better scrapping and searching algorithms.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.