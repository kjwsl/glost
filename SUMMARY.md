# Glost Language Learning Platform

Glost has been transformed into a comprehensive language learning application with the following features:

## Features Implemented

### 1. Interactive Reading Interface
- **PDF Support**: Read PDF documents directly in the browser.
- **Contextual Lookup**: Select any word or phrase to get instant definitions from Kaikki.org.
- **Flashcard Creation**: One-click addition of words to your flashcard deck, capturing the original sentence context.

### 2. Spaced Repetition System (SRS)
- **Flashcard Management**: View all your saved words with definitions and context.
- **Review Mode**: A dedicated study interface using the SuperMemo-2 algorithm to schedule reviews efficiently.
- **Progress Tracking**: Cards are scheduled based on your performance (Again, Hard, Good, Easy).

### 3. Gamification & Stats
- **Dashboard**: Track your daily streaks, total XP, words learned, and current level.
- **Achievements**: Unlock badges for milestones like "First Word" or "Week Warrior".
- **Activity Graph**: Visualize your learning consistency.

## Architecture

- **Backend (Rust)**:
  - `axum` web server handling API requests.
  - `kaikki` integration for dictionary lookups.
  - JSON-based persistence for flashcards with SRS metadata.
  - PDF/EPUB file serving.

- **Frontend (Next.js)**:
  - Modern React UI with Tailwind CSS.
  - `react-pdf` for document rendering.
  - `lucide-react` for iconography.
  - Client-side state management for interactive features.

## How to Run

1. **Start the Backend**:
   ```bash
   cargo run -- serve --port 8000 --dir .
   ```

2. **Start the Frontend**:
   ```bash
   cd web
   npm run dev
   ```

3. **Open Application**:
   Navigate to `http://localhost:3000` to start learning!

## Next Steps

- **User Accounts**: Implement proper authentication.
- **Audio Support**: Add text-to-speech for pronunciation.
- **Mobile App**: Adapt the UI for mobile using React Native or PWA features.
- **Social Features**: Add leaderboards and friend challenges.
