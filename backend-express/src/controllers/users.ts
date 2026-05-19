import { jsonOk, jsonEmptyOk, jsonErr, ErrorReason } from '../utils/response';
import { Response } from 'express';
import { db } from '../db';
import { AuthRequest } from '../middleware/auth';

export const getUsers = async (req: AuthRequest, res: Response) => {
    // Assuming matching users by ids or something
    const { user_ids } = req.body; // or whatever payload rust accepts
    // Rust accepts Json<GetUsersDto> maybe it's array of emails or ids
    
    let query = db.selectFrom('User').select(['id', 'email', 'username', 'created_at']);
    if (user_ids && user_ids.length > 0) {
        query = query.where('id', 'in', user_ids);
    }
    const users = await query.limit(10).execute();
    jsonOk(res, users);
};

export const addUserToProject = async (req: AuthRequest, res: Response) => {
    const { email, project_id, is_admin } = req.body;
    
    const user = await db.selectFrom('User').select('id').where('email', '=', email).executeTakeFirst();
    if (!user) return res.status(404).json({ error: 'User not found' });

    await db.insertInto('ProjectUser')
        .values({
            project_id: project_id,
            user_id: user.id,
            is_admin: is_admin,
            updated_at: new Date()
        })
        .onConflict((oc) => oc.constraint('unique_admin_per_project').doNothing())
        .execute();
    jsonEmptyOk(res);
};

export const getUsersInProject = async (req: AuthRequest, res: Response) => {
    const projectId = parseInt(req.params.project_id as string);
    
    const users = await db.selectFrom('ProjectUser as pu')
        .innerJoin('User as u', 'u.id', 'pu.user_id')
        .select(['u.id', 'u.email', 'u.username', 'pu.is_admin'])
        .where('pu.project_id', '=', projectId)
        .execute();
        
    jsonOk(res, users);
};
