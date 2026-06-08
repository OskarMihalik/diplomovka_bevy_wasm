import { jsonOk, jsonEmptyOk, jsonErr, ErrorReason } from '../utils/response';
import { Response } from 'express';
import { db } from '../db';
import { AuthRequest } from '../middleware/auth';

export const getTagMessages = async (req: AuthRequest, res: Response) => {
    const tagId = parseInt(req.params.tag_id as string);
    const messages = await db.selectFrom('TagMessage as tm')
        .innerJoin('User as u', 'u.id', 'tm.created_by_id')
        .select(['tm.id', 'tm.text', 'tm.created_at', 'tm.tag_id', 'tm.created_by_id', 'u.username'])
        .where('tm.tag_id', '=', tagId)
        // Check rust for sort/limit, we can order by created_at asc
        .orderBy('tm.created_at asc')
        .execute();

    jsonOk(res, messages);
};

export const createTagMessage = async (req: AuthRequest, res: Response) => {
    const { text, tag_id } = req.body;
    
    await db.insertInto('TagMessage')
        .values({
            text,
            tag_id,
            created_by_id: req.user!.id,
            updated_at: new Date()
        })
        .execute();

    const messages = await db.selectFrom('TagMessage as tm')
        .innerJoin('User as u', 'u.id', 'tm.created_by_id')
        .select(['tm.id', 'tm.text', 'tm.created_at', 'tm.tag_id', 'tm.created_by_id', 'u.username'])
        .where('tm.tag_id', '=', tag_id)
        .orderBy('tm.created_at asc')
        .execute();

    jsonOk(res, messages);
};
