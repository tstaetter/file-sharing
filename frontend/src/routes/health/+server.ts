import { json } from '@sveltejs/kit';

/** Health check endpoint for Koyeb and other orchestrators.
 * Returns 200 OK with {"status":"ok"}. */
export function GET() {
    return json({ status: 'ok' });
}
