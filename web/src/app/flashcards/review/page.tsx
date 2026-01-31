"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { ArrowLeft, Check, Volume2 } from "lucide-react";

type Flashcard = {
  id: string;
  word: string;
  definition: string;
  context?: string;
  source?: string;
  added_at: string;
  next_review_at: string;
};

export default function ReviewPage() {
  const router = useRouter();
  const [cards, setCards] = useState<Flashcard[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [showAnswer, setShowAnswer] = useState(false);
  const [loading, setLoading] = useState(true);
  const [finished, setFinished] = useState(false);

  useEffect(() => {
    fetch("http://localhost:8000/api/flashcards")
      .then((res) => res.json())
      .then((data: Flashcard[]) => {
        // Filter cards due for review
        const now = new Date();
        const due = data.filter(c => new Date(c.next_review_at) <= now);
        setCards(due);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to fetch flashcards:", err);
        setLoading(false);
      });
  }, []);

  const handleReview = async (quality: number) => {
    const card = cards[currentIndex];
    try {
        await fetch("http://localhost:8000/api/flashcards/review", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                card_id: card.id,
                quality
            })
        });

        if (currentIndex < cards.length - 1) {
            setCurrentIndex(prev => prev + 1);
            setShowAnswer(false);
        } else {
            setFinished(true);
        }
    } catch (e) {
        console.error("Failed to submit review:", e);
    }
  };

  const playAudio = (text: string) => {
    if (!window.speechSynthesis) return;
    const utterance = new SpeechSynthesisUtterance(text);
    window.speechSynthesis.speak(utterance);
  };

  if (loading) {
      return (
          <div className="flex h-full items-center justify-center">
              <div className="animate-pulse text-zinc-400">Loading reviews...</div>
          </div>
      );
  }

  if (finished) {
      return (
          <div className="flex h-full flex-col items-center justify-center space-y-6 text-center">
              <div className="rounded-full bg-green-100 p-6 text-green-600 dark:bg-green-900/30">
                  <Check className="h-12 w-12" />
              </div>
              <h2 className="text-2xl font-bold">All caught up!</h2>
              <p className="text-zinc-500">You&apos;ve reviewed all your due cards for now.</p>
              <button 
                onClick={() => router.push('/flashcards')}
                className="rounded-md bg-blue-600 px-6 py-2 text-white hover:bg-blue-700"
              >
                  Back to Flashcards
              </button>
          </div>
      );
  }

  if (cards.length === 0) {
      return (
          <div className="flex h-full flex-col items-center justify-center space-y-6 text-center">
              <div className="rounded-full bg-blue-100 p-6 text-blue-600 dark:bg-blue-900/30">
                  <Check className="h-12 w-12" />
              </div>
              <h2 className="text-2xl font-bold">No cards due</h2>
              <p className="text-zinc-500">Great job! You have no cards to review right now.</p>
              <button 
                onClick={() => router.push('/flashcards')}
                className="rounded-md bg-blue-600 px-6 py-2 text-white hover:bg-blue-700"
              >
                  Back to Flashcards
              </button>
          </div>
      );
  }

  const card = cards[currentIndex];

  return (
    <div className="flex flex-col h-full max-w-2xl mx-auto py-8">
      <div className="flex items-center justify-between mb-8">
          <button onClick={() => router.back()} className="p-2 hover:bg-zinc-100 rounded-full">
              <ArrowLeft className="w-5 h-5" />
          </button>
          <span className="text-sm font-medium text-zinc-500">
              {currentIndex + 1} / {cards.length}
          </span>
          <div className="w-9" /> {/* Spacer */}
      </div>

      <div className="flex-1 flex flex-col items-center justify-center space-y-8">
          {/* Card Face */}
          <div className="w-full bg-white dark:bg-zinc-900 rounded-xl shadow-lg border dark:border-zinc-800 p-12 text-center min-h-[300px] flex flex-col items-center justify-center relative">
                <div className="flex items-center gap-3 mb-4">
                    <h2 className="text-4xl font-bold">{card.word}</h2>
                    <button 
                        onClick={(e) => { e.stopPropagation(); playAudio(card.word); }}
                        className="p-2 bg-zinc-100 hover:bg-blue-100 text-zinc-500 hover:text-blue-600 rounded-full transition-colors"
                        title="Listen"
                    >
                        <Volume2 className="w-6 h-6" />
                    </button>
                </div>
                {card.context && (
                    <p className="text-zinc-500 italic text-sm mt-4 max-w-md">
                        &quot;{card.context}&quot;
                    </p>
                )}
                
                {showAnswer && (
                    <div className="mt-8 pt-8 border-t w-full animate-in fade-in slide-in-from-bottom-4">
                        <p className="text-xl text-zinc-800 dark:text-zinc-200">
                            {card.definition}
                        </p>
                        <p className="text-xs text-zinc-400 mt-4">
                            Source: {card.source || "Unknown"}
                        </p>
                    </div>
                )}
          </div>

          {/* Controls */}
          <div className="w-full">
              {!showAnswer ? (
                  <button 
                    onClick={() => setShowAnswer(true)}
                    className="w-full py-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium text-lg shadow-md transition-colors"
                  >
                      Show Answer
                  </button>
              ) : (
                  <div className="grid grid-cols-4 gap-4">
                      <button 
                        onClick={() => handleReview(1)}
                        className="flex flex-col items-center p-4 bg-red-100 hover:bg-red-200 text-red-700 rounded-lg transition-colors"
                      >
                          <span className="font-bold mb-1">Again</span>
                          <span className="text-xs opacity-70">&lt; 1m</span>
                      </button>
                      <button 
                        onClick={() => handleReview(3)}
                        className="flex flex-col items-center p-4 bg-orange-100 hover:bg-orange-200 text-orange-700 rounded-lg transition-colors"
                      >
                          <span className="font-bold mb-1">Hard</span>
                          <span className="text-xs opacity-70">2d</span>
                      </button>
                      <button 
                        onClick={() => handleReview(4)}
                        className="flex flex-col items-center p-4 bg-blue-100 hover:bg-blue-200 text-blue-700 rounded-lg transition-colors"
                      >
                          <span className="font-bold mb-1">Good</span>
                          <span className="text-xs opacity-70">4d</span>
                      </button>
                      <button 
                        onClick={() => handleReview(5)}
                        className="flex flex-col items-center p-4 bg-green-100 hover:bg-green-200 text-green-700 rounded-lg transition-colors"
                      >
                          <span className="font-bold mb-1">Easy</span>
                          <span className="text-xs opacity-70">7d</span>
                      </button>
                  </div>
              )}
          </div>
      </div>
    </div>
  );
}
