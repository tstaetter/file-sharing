import { Resend } from 'resend';
import { env } from '$env/dynamic/private';
import { PUBLIC_PREFIX } from '$env/static/public';
import { createElement } from 'react';
import { FileSharedTemplate } from '$lib/email/file-shared-template';
import type { RequestHandler } from './$types';

function getResend(): Resend | null {
	const key = env.RESEND_API_KEY;
	if (!key) {
		console.error('RESEND_API_KEY is not set — email sending is disabled');
		return null;
	}
	return new Resend(key);
}

interface SendBody {
	to: string;
	link: string;
	fileName?: string;
}

export const POST: RequestHandler = async ({ request }) => {
	let body: SendBody;
	try {
		body = await request.json();
	} catch {
		return Response.json({ error: 'Invalid JSON body' }, { status: 400 });
	}

	if (!body.to || !body.link) {
		return Response.json({ error: 'Missing required fields: to, link' }, { status: 400 });
	}

	if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(body.to)) {
		return Response.json({ error: 'Invalid email address' }, { status: 400 });
	}

	const appName = 'filez.zone';
	const appUrl = PUBLIC_PREFIX || 'https://filez.zone';

	const resend = getResend();
	if (!resend) {
		return Response.json({ error: 'Email service is not configured' }, { status: 503 });
	}

	const { data, error } = await resend.emails.send({
		from: `${appName} <no-reply@filez.zone>`,
		to: [body.to],
		subject: `${body.fileName || 'A file'} was shared with you via ${appName}`,
		react: createElement(FileSharedTemplate, {
			fileName: body.fileName || 'a file',
			link: body.link,
			appName,
			appUrl
		})
	});

	if (error) {
		console.error('Resend error:', error);
		return Response.json({ error: 'Failed to send email' }, { status: 500 });
	}

	return Response.json({ success: true, id: data?.id });
};
