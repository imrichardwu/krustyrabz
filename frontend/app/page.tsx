"use client";
/* eslint-disable @next/next/no-img-element */

import Link from "next/link";
import { useEffect, useState } from "react";
import { getSession, signOut } from "@/lib/supabase";
import { useRouter } from "next/navigation";

export default function Home() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [username, setUsername] = useState("");
  const [loading, setLoading] = useState(true);
  const router = useRouter();

  useEffect(() => {
    checkSession();
  }, []);

  const checkSession = async () => {
    try {
      const session = await getSession();
      if (session) {
        setIsLoggedIn(true);
        setUsername(session.user.user_metadata?.username || session.user.email);
      }
    } catch (error) {
      console.error("Session check failed:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleSignOut = async () => {
    try {
      await signOut();
      setIsLoggedIn(false);
      setUsername("");
    } catch (error) {
      console.error("Sign out failed:", error);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#0a0a0a]">
        <div className="flex flex-col items-center gap-4">
          <div className="w-10 h-10 border border-[#c4c5ca] border-t-transparent rounded-full animate-spin" />
          <p className="text-[#787878] text-sm tracking-widest uppercase">
            Loading
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen text-[#f0ede8]">
      {/* Navigation */}
      <nav
        className="fixed w-full top-0 z-50 bg-[#080a14]/92 backdrop-blur-md border-b border-[#c0102a]/25"
        style={{
          boxShadow:
            "0 1px 0 rgba(200,144,154,0.08), 0 4px 24px rgba(0,0,0,0.5)",
        }}
      >
        <div className="max-w-7xl mx-auto px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center gap-3">
              {/* Joker card logo */}
              <svg
                width="26"
                height="32"
                viewBox="0 0 26 32"
                fill="none"
                aria-hidden="true"
              >
                {/* Card body */}
                <rect
                  x="1"
                  y="1"
                  width="22"
                  height="30"
                  rx="3"
                  fill="#07080f"
                  stroke="#d4a520"
                  strokeWidth="1.3"
                />
                {/* 5-point star — centred at (12, 15.5) */}
                <path
                  d="M12 9.5 L13.3 13.5 L17.5 13.5 L14.1 15.9 L15.4 19.9 L12 17.5 L8.6 19.9 L9.9 15.9 L6.5 13.5 L10.7 13.5 Z"
                  fill="#d4a520"
                />
                {/* Top-left corner diamond */}
                <path
                  d="M4.5 3.5 L5.8 5.5 L4.5 7.5 L3.2 5.5 Z"
                  fill="#c0102a"
                />
                {/* Bottom-right corner diamond */}
                <path
                  d="M19.5 24.5 L20.8 26.5 L19.5 28.5 L18.2 26.5 Z"
                  fill="#c0102a"
                />
              </svg>
              <span className="text-xl font-bold tracking-tight text-[#f0ede8]">
                Light Poker
              </span>
            </div>

            <div className="flex items-center gap-3">
              {isLoggedIn ? (
                <>
                  <span className="text-[#787878] text-sm hidden sm:inline">
                    Welcome, {username}
                  </span>
                  <button
                    onClick={handleSignOut}
                    className="px-4 py-2 text-sm cs-btn-danger rounded-lg"
                  >
                    Sign Out
                  </button>
                  <button
                    onClick={() => router.push("/game")}
                    className="px-5 py-2 text-sm cs-btn rounded-lg"
                  >
                    Enter Game
                  </button>
                </>
              ) : (
                <>
                  <Link
                    href="/login"
                    className="px-4 py-2 text-sm text-[#787878] hover:text-[#c4c5ca] transition-colors duration-150"
                  >
                    Sign In
                  </Link>
                  <Link
                    href="/register"
                    className="px-5 py-2 text-sm cs-btn rounded-lg"
                  >
                    Create Account
                  </Link>
                </>
              )}
            </div>
          </div>
        </div>
      </nav>

      {/* Hero */}
      <section className="relative pt-36 pb-28 px-6 lg:px-8 overflow-hidden">
        <div
          className="absolute inset-0 pointer-events-none"
          aria-hidden="true"
        >
          {/* Casino atmosphere glows */}
          <div className="absolute top-0 left-1/4 w-150 h-125 -translate-x-1/2 rounded-full bg-[#c0102a]/10 blur-[120px]" />
          <div className="absolute top-0 right-1/4 w-125 h-100 translate-x-1/4 rounded-full bg-[#d4a520]/8 blur-[100px]" />
          <div className="absolute top-0 left-1/2 -translate-x-1/2 w-px h-full bg-linear-to-b from-transparent via-white/6 to-transparent" />
        </div>
        <div className="max-w-5xl mx-auto text-center relative z-10">
          <p className="uppercase tracking-[0.45em] text-xs text-[#d4a520] mb-6 font-semibold">
            The Art of the Game
          </p>
          <h1 className="text-5xl sm:text-6xl lg:text-7xl font-extrabold text-[#f0ede8] mb-6 leading-tight tracking-tight">
            All In{" "}
            <span
              style={{
                background:
                  "linear-gradient(120deg, #e01530 0%, #d4a520 50%, #f0c84a 100%)",
                WebkitBackgroundClip: "text",
                WebkitTextFillColor: "transparent",
              }}
            >
              or Nothing
            </span>
          </h1>
          <p className="text-lg sm:text-xl text-[#787878] mb-12 max-w-2xl mx-auto leading-relaxed">
            Powered by Rust for blazing speed — every hand dealt with surgical
            precision and razor-sharp strategy.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            {isLoggedIn ? (
              <button
                onClick={() => router.push("/game")}
                className="px-8 py-4 text-base font-bold cs-btn rounded-xl shadow-lg hover:scale-[1.02] transform transition-transform"
              >
                Enter the Arena
              </button>
            ) : (
              <>
                <Link
                  href="/register"
                  className="px-8 py-4 text-base font-bold cs-btn rounded-xl shadow-lg hover:scale-[1.02] transform transition-transform"
                >
                  Claim Your Seat
                </Link>
                <Link
                  href="/login"
                  className="px-8 py-4 text-base font-semibold cs-btn-ghost rounded-xl hover:scale-[1.02] transform transition-transform"
                >
                  Sign In
                </Link>
              </>
            )}
          </div>
        </div>
      </section>

      <div className="mx-6 lg:mx-8 cs-rule" />

      {/* Game Modes */}
      <section className="py-24 px-6 lg:px-8 relative overflow-hidden">
        <div
          className="absolute inset-0 pointer-events-none"
          aria-hidden="true"
        >
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-225 h-100 rounded-full bg-[#c8909a]/4 blur-[120px]" />
        </div>
        <div className="max-w-7xl mx-auto relative z-10">
          <div className="text-center mb-16">
            <p className="uppercase tracking-[0.3em] text-xs text-[#c0102a] mb-3 font-semibold">
              Choose Your Game
            </p>
            <h2 className="text-3xl sm:text-4xl font-bold text-[#f0ede8]">
              Three Ways to Play
            </h2>
          </div>

          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Five Card Draw — FEATURED */}
            <div
              className="cs-surface rounded-2xl p-8 flex flex-col hover:-translate-y-1 transition-transform duration-200 border border-[#c0102a]/40"
              style={{
                boxShadow:
                  "0 0 0 1px rgba(192,16,42,0.15), 0 20px 60px rgba(0,0,0,0.85), 0 0 40px rgba(192,16,42,0.08)",
              }}
            >
              <div className="flex items-start justify-between mb-6">
                <div className="p-3 rounded-lg bg-white/5 border border-white/10">
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="#c4c5ca"
                    strokeWidth="1.75"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <rect x="2" y="4" width="12" height="16" rx="2" />
                    <path d="M6 8h4M6 12h4M6 16h2" />
                    <rect
                      x="10"
                      y="2"
                      width="12"
                      height="16"
                      rx="2"
                      opacity="0.35"
                    />
                  </svg>
                </div>
                <span className="text-xs font-semibold px-2.5 py-1 rounded-full bg-[#c0102a]/12 text-[#e01530] border border-[#c0102a]/30 tracking-wide uppercase">
                  Available
                </span>
              </div>
              <h3 className="text-xl font-bold text-[#f0ede8] mb-3">
                Five Card Draw
              </h3>
              <p className="text-[#787878] text-sm leading-relaxed mb-6 flex-1">
                The original frontier game. Each player receives five private
                cards, draws to improve, and bets to win. Simple rules, deep
                strategy.
              </p>
              <div className="flex items-center justify-between">
                <span className="text-xs text-[#787878] font-medium">
                  2 – 6 Players
                </span>
                {isLoggedIn && (
                  <button
                    onClick={() => router.push("/game")}
                    className="text-xs font-semibold px-3 py-1.5 cs-btn rounded-lg"
                  >
                    Play Now
                  </button>
                )}
              </div>
            </div>

            {/* Texas Hold'em — ACTIVE */}
            <div className="cs-surface rounded-2xl p-8 flex flex-col hover:-translate-y-1 transition-transform duration-200 border border-[#c8909a]/20">
              <div className="flex items-start justify-between mb-6">
                <div className="p-3 rounded-lg bg-white/5 border border-white/10">
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="#c4c5ca"
                    strokeWidth="1.75"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <circle cx="12" cy="12" r="10" />
                    <path d="M2 12h20M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" />
                  </svg>
                </div>
                <span className="text-xs font-semibold px-2.5 py-1 rounded-full bg-[#c0102a]/12 text-[#e01530] border border-[#c0102a]/30 tracking-wide uppercase">
                  Available
                </span>
              </div>
              <h3 className="text-xl font-bold text-[#f0ede8] mb-3">
                Texas Hold'em
              </h3>
              <p className="text-[#787878] text-sm leading-relaxed mb-6 flex-1">
                Two hole cards, five community cards. The world's most-watched
                variant — community, strategy, and all-in moments on every
                street.
              </p>
              <div className="flex items-center justify-between">
                <span className="text-xs text-[#787878] font-medium">
                  2 – 10 Players
                </span>
                {isLoggedIn && (
                  <button
                    onClick={() => router.push("/game")}
                    className="text-xs font-semibold px-3 py-1.5 cs-btn rounded-lg"
                  >
                    Play Now
                  </button>
                )}
              </div>
            </div>

            {/* Seven Card Stud — ACTIVE */}
            <div className="cs-surface rounded-2xl p-8 flex flex-col hover:-translate-y-1 transition-transform duration-200 border border-[#c8909a]/20">
              <div className="flex items-start justify-between mb-6">
                <div className="p-3 rounded-lg bg-white/5 border border-white/10">
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="#c4c5ca"
                    strokeWidth="1.75"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M12 2L2 7l10 5 10-5-10-5z" />
                    <path d="M2 17l10 5 10-5M2 12l10 5 10-5" />
                  </svg>
                </div>
                <span className="text-xs font-semibold px-2.5 py-1 rounded-full bg-[#c0102a]/12 text-[#e01530] border border-[#c0102a]/30 tracking-wide uppercase">
                  Available
                </span>
              </div>
              <h3 className="text-xl font-bold text-[#f0ede8] mb-3">
                Seven Card Stud
              </h3>
              <p className="text-[#787878] text-sm leading-relaxed mb-6 flex-1">
                Old-school mastery. Seven cards across four betting rounds —
                three down, four up. Memory, reads, and nerve at every decision
                point.
              </p>
              <div className="flex items-center justify-between">
                <span className="text-xs text-[#787878] font-medium">
                  2 – 8 Players
                </span>
                {isLoggedIn && (
                  <button
                    onClick={() => router.push("/game")}
                    className="text-xs font-semibold px-3 py-1.5 cs-btn rounded-lg"
                  >
                    Play Now
                  </button>
                )}
              </div>
            </div>
          </div>
        </div>
      </section>

      <div className="mx-6 lg:mx-8 cs-rule" />

      {/* Why Light Poker */}
      <section className="py-24 px-6 lg:px-8 bg-[#0a0c18]/60 relative overflow-hidden">
        {/* subtle crimson casino glow behind features */}
        <div
          className="absolute inset-0 pointer-events-none"
          aria-hidden="true"
        >
          <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-200 h-75 rounded-full bg-[#c0102a]/6 blur-[100px]" />
        </div>
        <div className="max-w-7xl mx-auto relative z-10">
          <div className="text-center mb-16">
            <p className="uppercase tracking-[0.3em] text-xs text-[#c0102a] mb-3 font-semibold">
              Built Different
            </p>
            <h2 className="text-3xl sm:text-4xl font-bold text-[#f0ede8]">
              Why Players Choose Light Poker
            </h2>
          </div>

          <div className="grid md:grid-cols-3 gap-10">
            <div className="flex flex-col items-start gap-4">
              <div className="p-3 rounded-xl bg-[#c0102a]/10 border border-[#c0102a]/20">
                <svg
                  width="22"
                  height="22"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="#e01530"
                  strokeWidth="1.75"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
                </svg>
              </div>
              <h3 className="text-base font-bold text-[#f0ede8]">
                Rust-Powered Speed
              </h3>
              <p className="text-[#787878] text-sm leading-relaxed">
                Sub-millisecond response times. No garbage collection pauses, no
                lag — just clean, instant gameplay even under load.
              </p>
            </div>

            <div className="flex flex-col items-start gap-4">
              <div className="p-3 rounded-xl bg-[#c8909a]/10 border border-[#c8909a]/20">
                <svg
                  width="22"
                  height="22"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="#e0b4bc"
                  strokeWidth="1.75"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                </svg>
              </div>
              <h3 className="text-base font-bold text-[#f0ede8]">
                Secure & Authenticated
              </h3>
              <p className="text-[#787878] text-sm leading-relaxed">
                Protected routes, authenticated sessions, and server-validated
                game state keep every hand fair and every account safe.
              </p>
            </div>

            <div className="flex flex-col items-start gap-4">
              <div className="p-3 rounded-xl bg-[#c4c5ca]/10 border border-[#c4c5ca]/20">
                <svg
                  width="22"
                  height="22"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="#dcdde2"
                  strokeWidth="1.75"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                >
                  <circle cx="9" cy="7" r="4" />
                  <path d="M3 21v-2a4 4 0 014-4h4a4 4 0 014 4v2" />
                  <circle cx="19" cy="7" r="2.5" />
                  <path d="M23 21v-1a2 2 0 00-2-2h-1" />
                </svg>
              </div>
              <h3 className="text-base font-bold text-[#f0ede8]">
                Live Multiplayer
              </h3>
              <p className="text-[#787878] text-sm leading-relaxed">
                Create a table, share the game ID, and compete in real time. The
                arena is open around the clock.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Banner */}
      {!isLoggedIn && (
        <>
          <div className="mx-6 lg:mx-8 cs-rule" />
          <section className="py-24 px-6 lg:px-8">
            <div
              className="max-w-3xl mx-auto rounded-3xl p-14 text-center border border-[#c0102a]/30 relative overflow-hidden"
              style={{
                background:
                  "linear-gradient(160deg, rgba(14,16,30,0.98), rgba(8,10,20,0.99))",
                boxShadow:
                  "0 0 80px rgba(192,16,42,0.12), 0 0 60px rgba(212,165,32,0.06), 0 24px 60px rgba(0,0,0,0.9)",
              }}
            >
              {/* glow orb inside CTA */}
              <div className="absolute top-0 left-1/2 -translate-x-1/2 w-100 h-50 rounded-full bg-[#c0102a]/10 blur-[60px] pointer-events-none" />
              <div className="relative z-10">
                <p className="uppercase tracking-[0.35em] text-xs text-[#d4a520] mb-4 font-semibold">
                  Ready to Play
                </p>
                <h2 className="text-3xl sm:text-4xl font-bold text-[#f0ede8] mb-5">
                  Your Seat at the Table
                  <br className="hidden sm:block" /> is Waiting
                </h2>
                <p className="text-[#a0a09c] mb-10 leading-relaxed max-w-xl mx-auto">
                  Create a free account and start your first hand tonight. No
                  deposit required.
                </p>
                <div className="flex flex-col sm:flex-row gap-4 justify-center">
                  <Link
                    href="/register"
                    className="px-8 py-4 text-base font-bold cs-btn rounded-xl hover:scale-[1.02] transform transition-transform"
                  >
                    Create Free Account
                  </Link>
                  <Link
                    href="/login"
                    className="px-8 py-4 text-base font-semibold cs-btn-ghost rounded-xl hover:scale-[1.02] transform transition-transform"
                  >
                    Sign In
                  </Link>
                </div>
              </div>
            </div>
          </section>
        </>
      )}

      {/* Footer */}
      <footer className="py-8 px-6 lg:px-8 bg-[#05070f] border-t border-[#c0102a]/20">
        <div className="max-w-7xl mx-auto flex flex-col sm:flex-row justify-between items-center gap-3 text-sm">
          <span className="text-[#d4a520]/60">&copy; 2026 Light Poker</span>
          <span className="text-[#444444]">Play responsibly.</span>
        </div>
      </footer>
    </div>
  );
}
