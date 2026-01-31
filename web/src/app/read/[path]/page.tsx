"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { Document, Page, pdfjs } from "react-pdf";
import { ChevronLeft, ChevronRight, ZoomIn, ZoomOut, Loader2, X, BookOpen, Plus, Volume2 } from "lucide-react";
import { useParams, useRouter } from "next/navigation";
import { createPortal } from "react-dom";

import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";

// Configure worker
pdfjs.GlobalWorkerOptions.workerSrc = `//unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

type Definition = {
  word: string;
  definition: string;
  context?: string;
};

export default function ReaderPage() {
  const params = useParams();
  const router = useRouter();
  const path = decodeURIComponent(params.path as string);
  
  const [numPages, setNumPages] = useState<number | null>(null);
  const [pageNumber, setPageNumber] = useState(1);
  const [scale, setScale] = useState(1.0);
  const [selection, setSelection] = useState<{ text: string; rect: DOMRect } | null>(null);
  const [definition, setDefinition] = useState<Definition | null>(null);
  const [loadingDefinition, setLoadingDefinition] = useState(false);
  
  const containerRef = useRef<HTMLDivElement>(null);

  function onDocumentLoadSuccess({ numPages }: { numPages: number }) {
    setNumPages(numPages);
  }

  const handleTextSelection = useCallback(() => {
    const selection = window.getSelection();
    if (!selection || selection.isCollapsed) {
      setSelection(null);
      return;
    }

    const text = selection.toString().trim();
    if (text.length === 0) return;

    const range = selection.getRangeAt(0);
    const rect = range.getBoundingClientRect();

    setSelection({
      text,
      rect,
    });
    
    // Auto-lookup (optional, or wait for click)
    lookupWord(text);
  }, []);

  const lookupWord = async (word: string) => {
    setLoadingDefinition(true);
    setDefinition(null);
    try {
      const res = await fetch(`http://localhost:8000/api/lookup/${encodeURIComponent(word)}`);
      if (res.ok) {
        const data = await res.json();
        // Assuming backend returns { definitions: [...] } or similar. 
        // Based on src/server.rs, it calls `get_from_kaikki`.
        // Let's assume it returns a list of definitions.
        // We'll just take the first one for now or the whole object.
        // Wait, looking at server.rs, it returns `Json(definitions)`.
        setDefinition({
            word,
            definition: data[0]?.senses[0]?.raw_glosses?.[0] || "No definition found.",
            context: "" 
        });
      } else {
          setDefinition({ word, definition: "Definition not found." });
      }
    } catch (err) {
      console.error(err);
      setDefinition({ word, definition: "Error fetching definition." });
    } finally {
      setLoadingDefinition(false);
    }
  };

  const playAudio = (text: string) => {
    if (!window.speechSynthesis) return;
    const utterance = new SpeechSynthesisUtterance(text);
    // utterance.lang = 'en-US'; // Could be dynamic
    window.speechSynthesis.speak(utterance);
  };

  const addToFlashcards = async () => {
    if (!definition) return;
    
    try {
        await fetch("http://localhost:8000/api/flashcards", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                word: definition.word,
                definition: definition.definition,
                context: selection?.text || "", // Use full selection as context?
                source: path
            })
        });
        alert("Added to flashcards!");
        setSelection(null);
        setDefinition(null);
    } catch (e) {
        console.error(e);
        alert("Failed to add flashcard");
    }
  };

  // Close popup on click outside
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
        // If clicking outside the selection popup
        // This is tricky with portals.
        // For now simple implementation:
        if (!(e.target as HTMLElement).closest('.definition-popup')) {
             // Maybe clear selection?
        }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  return (
    <div className="flex flex-col h-full" onMouseUp={handleTextSelection}>
      {/* Toolbar */}
      <div className="flex items-center justify-between p-4 border-b bg-white dark:bg-zinc-900 sticky top-0 z-10 shadow-sm">
        <button onClick={() => router.push('/')} className="p-2 hover:bg-zinc-100 rounded-md">
            <BookOpen className="w-5 h-5" />
        </button>
        
        <div className="flex items-center space-x-4">
          <button 
            disabled={pageNumber <= 1} 
            onClick={() => setPageNumber(p => p - 1)}
            className="p-2 hover:bg-zinc-100 rounded-md disabled:opacity-50"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <span className="text-sm font-medium">
            Page {pageNumber} of {numPages || '--'}
          </span>
          <button 
            disabled={pageNumber >= (numPages || 0)} 
            onClick={() => setPageNumber(p => p + 1)}
            className="p-2 hover:bg-zinc-100 rounded-md disabled:opacity-50"
          >
            <ChevronRight className="w-5 h-5" />
          </button>
        </div>

        <div className="flex items-center space-x-2">
            <button onClick={() => setScale(s => Math.max(0.5, s - 0.1))} className="p-2 hover:bg-zinc-100 rounded-md">
                <ZoomOut className="w-5 h-5" />
            </button>
            <span className="text-sm w-12 text-center">{Math.round(scale * 100)}%</span>
            <button onClick={() => setScale(s => Math.min(2.0, s + 0.1))} className="p-2 hover:bg-zinc-100 rounded-md">
                <ZoomIn className="w-5 h-5" />
            </button>
        </div>
      </div>

      {/* PDF Viewer */}
      <div className="flex-1 overflow-auto bg-zinc-100 dark:bg-zinc-950 p-8 flex justify-center relative" ref={containerRef}>
        <Document
          file={`http://localhost:8000/api/books/${encodeURIComponent(path)}`}
          onLoadSuccess={onDocumentLoadSuccess}
          className="shadow-lg"
        >
          <Page 
            pageNumber={pageNumber} 
            scale={scale} 
            renderTextLayer={true}
            renderAnnotationLayer={true}
            className="bg-white"
          />
        </Document>
      </div>

      {/* Definition Popup */}
      {selection && (
          createPortal(
            <div 
                className="definition-popup fixed z-50 bg-white dark:bg-zinc-800 shadow-xl rounded-lg border dark:border-zinc-700 w-80 p-4 transition-all"
                style={{ 
                    top: selection.rect.bottom + window.scrollY + 10,
                    left: Math.min(selection.rect.left + window.scrollX, window.innerWidth - 340) // Prevent overflow right
                }}
            >
                <div className="flex justify-between items-start mb-2">
                    <div className="flex items-center gap-2 overflow-hidden">
                        <h3 className="font-bold text-lg text-blue-600 truncate">{selection.text}</h3>
                        <button 
                            onClick={() => playAudio(selection.text)}
                            className="p-1 hover:bg-zinc-100 rounded-full text-zinc-500 hover:text-blue-600 shrink-0"
                            title="Listen"
                        >
                            <Volume2 className="w-4 h-4" />
                        </button>
                    </div>
                    <button onClick={() => { setSelection(null); setDefinition(null); }} className="text-zinc-400 hover:text-zinc-600 shrink-0 ml-2">
                        <X className="w-4 h-4" />
                    </button>
                </div>
                
                {loadingDefinition ? (
                    <div className="flex justify-center py-4">
                        <Loader2 className="w-6 h-6 animate-spin text-blue-600" />
                    </div>
                ) : definition ? (
                    <div className="space-y-3">
                        <p className="text-sm text-zinc-700 dark:text-zinc-300 max-h-32 overflow-y-auto">
                            {definition.definition}
                        </p>
                        <button 
                            onClick={addToFlashcards}
                            className="w-full flex items-center justify-center space-x-2 bg-blue-600 hover:bg-blue-700 text-white py-2 rounded-md text-sm font-medium transition-colors"
                        >
                            <Plus className="w-4 h-4" />
                            <span>Add to Flashcards</span>
                        </button>
                    </div>
                ) : (
                    <p className="text-sm text-zinc-500">Definition not found.</p>
                )}
            </div>,
            document.body
          )
      )}
    </div>
  );
}
