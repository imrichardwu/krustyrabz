"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { signIn } from "@/lib/supabase";

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      const session = await signIn(email, password);
      console.log("Login successful:", session);
      router.push("/");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-4 relative overflow-hidden bg-[#0a0a0a]">
      {/* Subtle corner accents */}
      <div className="absolute top-0 left-0 w-64 h-64 bg-white/2 rounded-br-full pointer-events-none" />
      <div className="absolute bottom-0 right-0 w-64 h-64 bg-[#c4c5ca]/2 rounded-tl-full pointer-events-none" />

      <div className="w-full max-w-md">
        {/* Back to Home */}
        <Link
          href="/"
          className="inline-flex items-center text-[#c4c5ca] hover:text-[#dcdde2] mb-8 transition-colors duration-150"
        >
          <svg
            className="w-5 h-5 mr-2"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M10 19l-7-7m0 0l7-7m-7 7h18"
            />
          </svg>
          Back to Home
        </Link>

        {/* Login Card */}
        <div className="cs-surface rounded-2xl p-8">
          <div className="text-center mb-8">
            <div className="w-10 h-px bg-linear-to-r from-transparent via-[#c0102a]/60 to-transparent mx-auto mb-6" />
            <h1 className="text-2xl font-bold text-[#f0ede8] mb-2">
              Welcome Back
            </h1>
            <p className="text-[#787878] text-sm tracking-wide">
              Sign in to Light Poker
            </p>
          </div>

          {error && (
            <div className="mb-6 p-4 bg-[#c0102a]/12 border border-[#c0102a]/35 rounded-lg text-[#ffd0d5] text-sm">
              {error}
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Email Field */}
            <div>
              <label
                htmlFor="email"
                className="block text-xs font-medium text-[#787878] mb-2 uppercase tracking-wider"
              >
                Email Address
              </label>
              <input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                className="w-full px-4 py-3 cs-input rounded-lg focus:outline-none transition-all duration-200"
                placeholder="you@example.com"
              />
            </div>

            {/* Password Field */}
            <div>
              <label
                htmlFor="password"
                className="block text-xs font-medium text-[#787878] mb-2 uppercase tracking-wider"
              >
                Password
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                minLength={6}
                className="w-full px-4 py-3 cs-input rounded-lg focus:outline-none transition-all duration-200"
                placeholder="••••••••"
              />
            </div>

            {/* Submit Button */}
            <button
              type="submit"
              disabled={loading}
              className="w-full py-3.5 cs-btn disabled:opacity-40 disabled:cursor-not-allowed rounded-lg transition-all duration-200 hover:scale-[1.01] transform"
            >
              {loading ? (
                <span className="flex items-center justify-center">
                  <svg
                    className="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    ></circle>
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  Logging in...
                </span>
              ) : (
                "Sign In"
              )}
            </button>
          </form>

          {/* Divider */}
          <div className="mt-8 pt-6 border-t border-white/8">
            <p className="text-center text-[#787878] text-sm">
              Don't have an account?{" "}
              <Link
                href="/register"
                className="text-[#c4c5ca] hover:text-[#dcdde2] font-semibold transition-colors duration-150"
              >
                Create one free
              </Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
