
import { cookies } from 'next/headers';

export type Ctx = {
    username: string;
    isAdmin: boolean;
    exp: Date;
};


function b64uDecode(str: string): string {
    return Buffer.from(str, 'base64url').toString('utf8');
}

function isFutureISO(dateStr: string): boolean {
    const d = new Date(dateStr);
    return !isNaN(d.getTime()) && d > new Date();
}


export async function getCtxFromCookie(): Promise<Ctx | null> {
    const cookieStore = await cookies();
    const raw = cookieStore.get('auth-token')?.value;
    if (!raw) return null;

    try {

        const [identifierEnc, expirationEnc] = raw.split(':');
        const username = b64uDecode(identifierEnc);
        const expStr = b64uDecode(expirationEnc);

        if (!isFutureISO(expStr)) return null;

        const isAdmin = /admin/i.test(username);

        return { username, isAdmin, exp: new Date(expStr) };
    } catch {
        return null;
    }
}
