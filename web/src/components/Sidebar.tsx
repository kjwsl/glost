"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { BookOpen, Library, Zap, Trophy, Settings } from "lucide-react";
import { clsx } from "clsx";

const navigation = [
  { name: "Library", href: "/", icon: Library },
  { name: "Flashcards", href: "/flashcards", icon: Zap },
  { name: "Stats & Progress", href: "/stats", icon: Trophy },
  { name: "Settings", href: "/settings", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <div className="flex h-screen w-64 flex-col border-r bg-white dark:bg-zinc-900 dark:border-zinc-800">
      <div className="flex h-16 items-center px-6 border-b dark:border-zinc-800">
        <BookOpen className="h-6 w-6 text-blue-600 mr-2" />
        <span className="text-xl font-bold">Glost</span>
      </div>
      <nav className="flex-1 space-y-1 px-4 py-4">
        {navigation.map((item) => {
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.name}
              href={item.href}
              className={clsx(
                isActive
                  ? "bg-zinc-100 text-blue-600 dark:bg-zinc-800 dark:text-blue-400"
                  : "text-zinc-600 hover:bg-zinc-50 dark:text-zinc-400 dark:hover:bg-zinc-800",
                "group flex items-center rounded-md px-3 py-2 text-sm font-medium transition-colors"
              )}
            >
              <item.icon
                className={clsx(
                  isActive ? "text-blue-600 dark:text-blue-400" : "text-zinc-400 group-hover:text-zinc-500",
                  "mr-3 h-5 w-5 flex-shrink-0"
                )}
              />
              {item.name}
            </Link>
          );
        })}
      </nav>
      <div className="p-4 border-t dark:border-zinc-800">
        <div className="flex items-center">
          <div className="h-8 w-8 rounded-full bg-blue-100 flex items-center justify-center text-blue-600 font-bold">
            U
          </div>
          <div className="ml-3">
            <p className="text-sm font-medium">User</p>
            <p className="text-xs text-zinc-500">Learner</p>
          </div>
        </div>
      </div>
    </div>
  );
}
