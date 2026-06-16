"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { getSession } from "@/lib/supabase";

export default function ProtectedLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    getSession().then((session) => {
      if (!session) {
        router.replace("/login");
      } else {
        setChecking(false);
      }
    });
  }, [router]);

  if (checking) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#0a0a0a]">
        <div className="flex flex-col items-center gap-3">
          <div className="w-10 h-10 border border-[#c4c5ca] border-t-transparent rounded-full animate-spin" />
          <p className="text-[#787878] text-sm tracking-widest uppercase">
            Verifying session
          </p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
