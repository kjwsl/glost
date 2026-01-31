"use client";

import { Trophy, Flame, Target, Star } from "lucide-react";

export default function StatsPage() {
  return (
    <div className="space-y-8">
      <header>
        <h1 className="text-3xl font-bold tracking-tight">Stats & Progress</h1>
        <p className="text-zinc-500 dark:text-zinc-400">
          Track your learning journey and achievements.
        </p>
      </header>

      {/* Top Stats */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-lg border bg-white p-6 shadow-sm dark:bg-zinc-900 dark:border-zinc-800">
            <div className="flex items-center justify-between">
                <div>
                    <p className="text-sm font-medium text-zinc-500">Daily Streak</p>
                    <p className="text-3xl font-bold text-orange-500">7 Days</p>
                </div>
                <div className="rounded-full bg-orange-100 p-3 text-orange-600">
                    <Flame className="w-6 h-6" />
                </div>
            </div>
        </div>

        <div className="rounded-lg border bg-white p-6 shadow-sm dark:bg-zinc-900 dark:border-zinc-800">
            <div className="flex items-center justify-between">
                <div>
                    <p className="text-sm font-medium text-zinc-500">Total XP</p>
                    <p className="text-3xl font-bold text-blue-600">1,250</p>
                </div>
                <div className="rounded-full bg-blue-100 p-3 text-blue-600">
                    <Star className="w-6 h-6" />
                </div>
            </div>
        </div>

        <div className="rounded-lg border bg-white p-6 shadow-sm dark:bg-zinc-900 dark:border-zinc-800">
            <div className="flex items-center justify-between">
                <div>
                    <p className="text-sm font-medium text-zinc-500">Words Learned</p>
                    <p className="text-3xl font-bold text-green-600">42</p>
                </div>
                <div className="rounded-full bg-green-100 p-3 text-green-600">
                    <Target className="w-6 h-6" />
                </div>
            </div>
        </div>

        <div className="rounded-lg border bg-white p-6 shadow-sm dark:bg-zinc-900 dark:border-zinc-800">
            <div className="flex items-center justify-between">
                <div>
                    <p className="text-sm font-medium text-zinc-500">Current Level</p>
                    <p className="text-3xl font-bold text-purple-600">Novice</p>
                </div>
                <div className="rounded-full bg-purple-100 p-3 text-purple-600">
                    <Trophy className="w-6 h-6" />
                </div>
            </div>
        </div>
      </div>

      {/* Achievements */}
      <div className="space-y-4">
        <h2 className="text-xl font-bold">Recent Achievements</h2>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div className="flex items-center space-x-4 rounded-lg border bg-white p-4 dark:bg-zinc-900 dark:border-zinc-800">
                <div className="rounded-full bg-yellow-100 p-3 text-yellow-600">
                    <Trophy className="w-6 h-6" />
                </div>
                <div>
                    <h3 className="font-bold">First Word</h3>
                    <p className="text-sm text-zinc-500">Saved your first word to flashcards.</p>
                </div>
            </div>
            <div className="flex items-center space-x-4 rounded-lg border bg-white p-4 dark:bg-zinc-900 dark:border-zinc-800 opacity-50 grayscale">
                <div className="rounded-full bg-zinc-100 p-3 text-zinc-600">
                    <Flame className="w-6 h-6" />
                </div>
                <div>
                    <h3 className="font-bold">Week Warrior</h3>
                    <p className="text-sm text-zinc-500">Reach a 7-day streak.</p>
                </div>
            </div>
        </div>
      </div>
      
      {/* Activity Graph Placeholder */}
      <div className="rounded-lg border bg-white p-6 dark:bg-zinc-900 dark:border-zinc-800">
        <h2 className="text-xl font-bold mb-4">Activity</h2>
        <div className="h-64 flex items-end space-x-2">
            {[30, 45, 20, 60, 75, 50, 80].map((h, i) => (
                <div key={i} className="flex-1 bg-blue-100 dark:bg-blue-900/30 rounded-t-sm relative group">
                    <div 
                        className="absolute bottom-0 w-full bg-blue-500 rounded-t-sm transition-all duration-500" 
                        style={{ height: `${h}%` }}
                    ></div>
                    <div className="absolute -bottom-6 w-full text-center text-xs text-zinc-500">
                        {['M', 'T', 'W', 'T', 'F', 'S', 'S'][i]}
                    </div>
                </div>
            ))}
        </div>
      </div>
    </div>
  );
}
