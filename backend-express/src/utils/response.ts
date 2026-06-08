import { Response } from 'express';

export const jsonOk = (res: Response, data: any) => {
    res.json({ Ok: data });
};

export const jsonEmptyOk = (res: Response) => {
    res.json({ Ok: null });
};

// Map reasons to the string representations defined in Rust enum `ErrorReason`
export enum ErrorReason {
    BadRequest = 'BadRequest',
    Unauthorized = 'Unauthorized',
    BadCredentials = 'BadCredentials',
    InvalidToken = 'InvalidToken',
}

export const jsonErr = (res: Response, reason: ErrorReason, message: string, status: number = 200) => {
    res.status(status).json({
        Err: {
            reason,
            message
        }
    });
};
