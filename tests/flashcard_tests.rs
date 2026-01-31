use glost::{FlashcardList, Flashcard};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_flashcard_list_new() {
    let list = FlashcardList::new();
    assert!(list.cards.is_empty());
}

#[test]
fn test_flashcard_list_add() {
    let mut list = FlashcardList::new();
    list.add(
        "test".to_string(),
        "definition".to_string(),
        Some("context".to_string()),
        Some("source".to_string()),
    );
    assert_eq!(list.cards.len(), 1);
    assert_eq!(list.cards[0].word, "test");
    assert_eq!(list.cards[0].definition, "definition");
    assert_eq!(list.cards[0].context, Some("context".to_string()));
    assert_eq!(list.cards[0].source, Some("source".to_string()));
    assert!(!list.cards[0].id.is_empty());
}

#[test]
fn test_flashcard_list_persistence() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("flashcards.json");

    let mut list = FlashcardList::new();
    list.add("word1".to_string(), "def1".to_string(), None, None);

    list.save(&file_path).unwrap();

    let loaded_list = FlashcardList::load(&file_path).unwrap();
    assert_eq!(loaded_list.cards.len(), 1);
    assert_eq!(loaded_list.cards[0].word, "word1");
}
