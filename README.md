# Glost - Glossary Generator

A command-line tool for generating glossaries from ebooks and documents, with language-specific filtering capabilities.

## Features

- **Multi-format Support**: Extract text from EPUB, PDF, and TXT files
- **Dictionary Integration**: Look up word definitions using the Kaikki.org API
- **Language-Specific Filtering**: Maintain separate filter lists for different languages
- **Markdown Output**: Generate glossaries in markdown format

## Installation

```bash
cargo build --release
```

## Usage

### Generate a Glossary

```bash
# Basic usage
glost generate book.epub

# Specify language and output file
glost generate --lang Swedish --output swedish_glossary.md book.epub

# Use custom filter file
glost generate --filter my_filters.txt book.epub
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

- Afrikaans, Chinese, Dutch, English, French, German
- Italian, Japanese, Korean, Mandarin, Portuguese
- Russian, Spanish, Swedish

## Code Structure

- `src/main.rs` - Entry point
- `src/cli.rs` - Command-line interface definitions
- `src/commands.rs` - Command handlers
- `src/content.rs` - File content extraction
- `src/filter.rs` - Filter list management
- `src/glossary.rs` - Glossary generation and output
- `src/kaikki/` - Kaikki.org API integration
- `src/language.rs` - Language definitions and utilities