import { redirect } from '@sveltejs/kit';

// NOTE: Redirect to /home by default
export const load = () => redirect(302, '/home');
