"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Book as BookIcon, FileText } from "lucide-react";

type Book = {
  name: string;
  path: string;
};

export default function Library() {
  const [books, setBooks] = useState<Book[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch("http://localhost:8000/api/books")
      .then((res) => res.json())
      .then((data) => {
        setBooks(data);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to fetch books:", err);
        setLoading(false);
      });
  }, []);

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-3xl font-bold tracking-tight">Library</h1>
        <p className="text-zinc-500 dark:text-zinc-400">
          Select a book to start reading and learning.
        </p>
      </header>

      {loading ? (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {[1, 2, 3].map((i) => (
            <div
              key={i}
              className="h-48 animate-pulse rounded-lg bg-zinc-200 dark:bg-zinc-800"
            />
          ))}
        </div>
      ) : books.length > 0 ? (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {books.map((book) => (
            <Link
              key={book.path.toString()}
              href={`/read/${encodeURIComponent(book.path.toString())}`}
              className="group relative flex flex-col justify-between overflow-hidden rounded-lg border bg-white p-6 shadow-sm transition-all hover:shadow-md dark:bg-zinc-900 dark:border-zinc-800"
            >
              <div className="flex items-start justify-between">
                <div className="rounded-full bg-blue-100 p-3 dark:bg-blue-900/30">
                  <BookIcon className="h-6 w-6 text-blue-600 dark:text-blue-400" />
                </div>
              </div>
              <div className="mt-4">
                <h3 className="text-lg font-medium group-hover:text-blue-600 transition-colors">
                  {book.name}
                </h3>
                <p className="mt-1 text-sm text-zinc-500">PDF Document</p>
              </div>
            </Link>
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center dark:border-zinc-800">
          <FileText className="h-12 w-12 text-zinc-400" />
          <h3 className="mt-4 text-lg font-medium">No books found</h3>
          <p className="mt-2 text-zinc-500">
            Add PDF or EPUB files to your glost directory to get started.
          </p>
        </div>
      )}
    </div>
  );
}
