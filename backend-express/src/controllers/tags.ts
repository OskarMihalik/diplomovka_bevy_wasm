import { jsonOk, jsonEmptyOk, jsonErr, ErrorReason } from '../utils/response';
import { Response } from 'express';
import { db } from '../db';
import { AuthRequest } from '../middleware/auth';

const getTagsForProject = async (projectId: number) => {
    const records = await db.selectFrom('Tag as t')
        .leftJoin('Status as s', 's.id', 't.status_id')
        .innerJoin('User as u', 'u.id', 't.created_by_id')
        .select([
            't.id', 't.title', 't.project_id', 't.created_at', 't.position_x', 't.position_y', 't.position_z',
            't.scale_x', 't.scale_y', 't.scale_z', 't.rotation_x', 't.rotation_y', 't.rotation_z', 't.created_by_id',
            'u.email as user_email', 'u.username as user_username',
            's.title as status_title', 's.id as status_id', 's.color_r as status_color_r', 's.color_g as status_color_g', 's.color_b as status_color_b', 's.project_id as status_project_id', 's.shape as status_shape'
        ])
        .where('t.project_id', '=', projectId)
        .limit(100)
        .offset(0)
        .execute();

    return records.map(tag => {
        let status_dto = null;
        if (tag.status_id !== null) {
            status_dto = {
                title: tag.status_title,
                id: tag.status_id,
                color_r: tag.status_color_r,
                color_g: tag.status_color_g,
                color_b: tag.status_color_b,
                project_id: tag.status_project_id,
                shape: tag.status_shape
            };
        }
        return {
            id: tag.id,
            title: tag.title,
            project_id: tag.project_id,
            created_at: tag.created_at,
            position_x: tag.position_x,
            position_y: tag.position_y,
            position_z: tag.position_z,
            created_by_id: tag.created_by_id,
            email: tag.user_email,
            username: tag.user_username,
            status_dto,
            scale_x: tag.scale_x,
            scale_y: tag.scale_y,
            scale_z: tag.scale_z,
            rotation_x: tag.rotation_x,
            rotation_y: tag.rotation_y,
            rotation_z: tag.rotation_z,
        };
    });
};

export const getTags = async (req: AuthRequest, res: Response) => {
    const projectId = parseInt(req.params.model_id as string); // In Rust it's named model_id in route but project_id logically
    const tags = await getTagsForProject(projectId);
    jsonOk(res, tags);
};

export const insertTag = async (req: AuthRequest, res: Response) => {
    const { title, project_id, position_x, position_y, position_z } = req.body;
    
    await db.insertInto('Tag')
        .values({
            title,
            project_id,
            position_x,
            position_y,
            position_z,
            created_by_id: req.user!.id,
            updated_at: new Date()
        })
        .execute();

    const tags = await getTagsForProject(project_id);
    jsonOk(res, tags);
};

export const updateTag = async (req: AuthRequest, res: Response) => {
    const { id, title, position_x, position_y, position_z, scale_x, scale_y, scale_z, rotation_x, rotation_y, rotation_z, status_dto, project_id } = req.body;

    await db.updateTable('Tag')
        .set({
            title,
            position_x, position_y, position_z,
            scale_x, scale_y, scale_z,
            rotation_x, rotation_y, rotation_z,
            status_id: status_dto ? status_dto.id : null,
            updated_at: new Date()
        })
        .where('id', '=', id)
        .execute();

    const tags = await getTagsForProject(project_id);
    jsonOk(res, tags);
};

export const deleteTag = async (req: AuthRequest, res: Response) => {
    const tagId = parseInt(req.params.tag_id as string);
    await db.deleteFrom('Tag').where('id', '=', tagId).execute();
    jsonEmptyOk(res);
};
