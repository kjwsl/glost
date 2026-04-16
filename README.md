# Glost - Glossary Generator

A command-line tool for generating glossaries from ebooks and documents, with language-specific filtering capabilities and AI-powered lemmatization.

## Features

- **Multi-format Support**: Extract text from EPUB, PDF, SRT, VTT, and TXT files
- **Web Article Support**: Fetch and process content directly from URLs
- **Dictionary Integration**: Look up word definitions using the Kaikki.org API
- **AI-Powered Lemmatization**: Use local AI (via Ollama) to find dictionary forms of inflected words (crucial for languages like Finnish)
- **Context Sentences**: Automatically extracts and includes the sentence where each word was found
- **Frequency Sorting**: Glossaries are sorted by word frequency in the source text
- **Language-Specific Filtering**: Maintain separate filter lists for different languages
- **Markdown Output**: Generate rich glossaries in markdown format with context blockquotes

## Installation

```bash
cargo build --release
```

## Usage

### Generate a Glossary

```bash
# Basic usage
glost generate book.epub

# Specify language and output file (supports .epub, .pdf, .srt, .vtt, .txt)
glost generate --lang Finnish --output finnish_glossary.md movie.srt

# Use local AI (Ollama) for lemmatization and better accuracy
# Ensure Ollama is running (e.g., 'ollama run llama3.2')
glost generate --ai-model llama3.2 book.epub
```

### Generate a Glossary from a Web Article
```bash
glost web https://yle.fi/uutiset/osasto/selkouutiset/ --lang Finnish --ai-model llama3.2
```

### Generate a Glossary from a YouTube Video
```bash
glost youtube https://www.youtube.com/watch?v=dQw4w9WgXcQ --lang English --ai-model llama3.2
```

### Manage Filter Lists

Filter lists allow you to exclude words you already know from the generated glossary.

```bash
# Add words to filter (defaults to English)
glost filter add the and it is was were

# Add words for specific language
glost filter add --lang Swedish och att det är

# List all filtered words
glost filter list

# List words for specific language
glost filter list --lang Swedish

# Remove words from filter
glost filter remove --lang English the and

# Clear words for specific language
glost filter clear --lang Swedish

# Clear all filter lists
glost filter clear
```

## Filter File Format

The filter file uses a simple format:
- English words: `word` (no prefix for backward compatibility)
- Other languages: `language:word`
- Comments: Lines starting with `#`

Example:
```
# Filter list - Format: language:word or just word (defaults to English)

and
is
the
Swedish:och
Swedish:att
Swedish:det
```

## Supported Languages

- Afrikaans
- Dutch
- English
- Finnish
- French
- German
- Italian
- Japanese
- Korean
- Mandarin
- Portuguese
- Russian
- Spanish
- Swedish

## Code Structure

- `src/main.rs` - Entry point
- `src/ai.rs` - Local AI integration (Ollama)
- `src/cli.rs` - Command-line interface definitions
- `src/commands.rs` - Command handlers
- `src/content.rs` - File content extraction and sentence segmentation
- `src/filter.rs` - Filter list management
- `src/glossary.rs` - Glossary generation and markdown formatting
- `src/kaikki/` - Kaikki.org API integration
- `src/language.rs` - Language definitions and utilities
