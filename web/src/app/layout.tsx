import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Sidebar } from "@/components/Sidebar";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Glost - Language Learning",
  description: "Interactive language learning with PDFs",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased flex h-screen bg-zinc-50 dark:bg-black text-zinc-900 dark:text-zinc-100`}
      >
        <Sidebar />
        <main className="flex-1 overflow-auto">
          <div className="container mx-auto p-8 max-w-7xl">
            {children}
          </div>
        </main>
      </body>
    </html>
  );
}
