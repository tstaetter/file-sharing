import { Resend } from 'resend';
import { RESEND_API_KEY } from '$env/static/private'; // define in your .env file
import type { RequestHandler } from './$types';

const resend = new Resend(RESEND_API_KEY);

export const POST: RequestHandler = async ({ request }) => {
	try {
		const { data, error } = await resend.emails.send({
			from: 'Acme <onboarding@resend.dev>',
			to: ['delivered@resend.dev'],
			subject: 'File shared',
			html: `<h3>Hi there</h3><p>A file has been shared with you: ${link}</p><footer>filez.zone</footer>`
		});

		if (error) {
			return Response.json({ error }, { status: 500 });
		}

		return Response.json({ data });
	} catch (error) {
		return Response.json({ error }, { status: 500 });
	}
};
