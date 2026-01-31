"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Zap, BookOpen, Calendar } from "lucide-react";

type Flashcard = {
  id: string;
  word: string;
  definition: string;
  context?: string;
  source?: string;
  added_at: string;
};

export default function FlashcardsPage() {
  const [cards, setCards] = useState<Flashcard[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch("http://localhost:8000/api/flashcards")
      .then((res) => res.json())
      .then((data) => {
        setCards(data);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to fetch flashcards:", err);
        setLoading(false);
      });
  }, []);

  return (
    <div className="space-y-6">
      <header className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Flashcards</h1>
          <p className="text-zinc-500 dark:text-zinc-400">
            Review your saved vocabulary.
          </p>
        </div>
        <Link 
          href="/flashcards/review"
          className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md font-medium flex items-center gap-2"
        >
            <Zap className="w-4 h-4" />
            Start Review
        </Link>
      </header>

      {loading ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {[1,2,3].map(i => (
                <div key={i} className="h-32 animate-pulse bg-zinc-200 dark:bg-zinc-800 rounded-lg"></div>
            ))}
        </div>
      ) : cards.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {cards.map((card) => (
            <div
              key={card.id}
              className="group relative flex flex-col justify-between rounded-lg border bg-white p-6 shadow-sm transition-all hover:shadow-md dark:bg-zinc-900 dark:border-zinc-800"
            >
              <div>
                <h3 className="text-xl font-bold text-zinc-900 dark:text-zinc-100 mb-2">
                  {card.word}
                </h3>
                <p className="text-sm text-zinc-600 dark:text-zinc-400 line-clamp-3">
                  {card.definition}
                </p>
                {card.context && (
                    <div className="mt-3 p-2 bg-zinc-50 dark:bg-zinc-800 rounded text-xs text-zinc-500 italic border dark:border-zinc-700">
                        &quot;{card.context}&quot;
                    </div>
                )}
              </div>
              
              <div className="mt-4 flex items-center justify-between text-xs text-zinc-400 border-t pt-4 dark:border-zinc-800">
                <div className="flex items-center gap-1">
                    <BookOpen className="w-3 h-3" />
                    <span className="truncate max-w-[100px]">{card.source || "Unknown source"}</span>
                </div>
                <div className="flex items-center gap-1">
                    <Calendar className="w-3 h-3" />
                    <span>{new Date(card.added_at).toLocaleDateString()}</span>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center dark:border-zinc-800">
          <Zap className="h-12 w-12 text-zinc-400" />
          <h3 className="mt-4 text-lg font-medium">No flashcards yet</h3>
          <p className="mt-2 text-zinc-500">
            Read books and select words to add them to your flashcards.
          </p>
        </div>
      )}
    </div>
  );
}
