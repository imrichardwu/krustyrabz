import { createClient } from "@supabase/supabase-js";

const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL!;
const supabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!;

export const supabase = createClient(supabaseUrl, supabaseAnonKey);

export interface AuthSession {
  access_token: string;
  refresh_token: string;
  user_id: string;
  email: string;
  username: string;
}

export async function signUp(
  email: string,
  password: string,
  username: string,
): Promise<AuthSession> {
  const { data, error } = await supabase.auth.signUp({
    email,
    password,
    options: {
      data: {
        username,
      },
    },
  });

  if (error) throw new Error(error.message);
  if (!data.user || !data.session) throw new Error("Registration failed");

  return {
    access_token: data.session.access_token,
    refresh_token: data.session.refresh_token,
    user_id: data.user.id,
    email: data.user.email!,
    username: data.user.user_metadata?.username || username,
  };
}

export async function signIn(
  email: string,
  password: string,
): Promise<AuthSession> {
  const { data, error } = await supabase.auth.signInWithPassword({
    email,
    password,
  });

  if (error) throw new Error(error.message);
  if (!data.user || !data.session) throw new Error("Login failed");

  return {
    access_token: data.session.access_token,
    refresh_token: data.session.refresh_token,
    user_id: data.user.id,
    email: data.user.email!,
    username: data.user.user_metadata?.username || "Player",
  };
}

export async function signOut() {
  const { error } = await supabase.auth.signOut();
  if (error) throw new Error(error.message);
}

export async function getSession() {
  const {
    data: { session },
  } = await supabase.auth.getSession();
  return session;
}
