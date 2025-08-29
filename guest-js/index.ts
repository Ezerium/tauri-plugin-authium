import { invoke } from '@tauri-apps/api/core'
import { User } from './types';

export async function getUser(): Promise<User | null> {
    return await invoke<User | null>('plugin:authium|get_user');
}

export async function isLoggedIn(): Promise<boolean> {
    return await invoke<boolean>('plugin:authium|is_logged_in');
}

export function signIn(expiry: number | null = 30 * 24 * 60 * 60): void {
    invoke('plugin:authium|sign_in', expiry == null ? undefined : { expiry });
}

export async function logout(): Promise<void> {
    await invoke('plugin:authium|logout');
}

export function refresh(refresh_data: boolean = false): void {
    invoke('plugin:authium|refresh', { refresh_data });
}

export * from './types';