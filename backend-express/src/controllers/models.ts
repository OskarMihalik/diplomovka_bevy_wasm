import { jsonOk, jsonEmptyOk, jsonErr, ErrorReason } from '../utils/response';
import { Response } from 'express';
import { db } from '../db';
import { AuthRequest } from '../middleware/auth';
import multer from 'multer';
import path from 'path';

export const getModel = async (req: AuthRequest, res: Response) => {
    const modelId = parseInt(req.params.model_id as string);
    const model = await db.selectFrom('Model')
        .selectAll()
        .where('id', '=', modelId)
        .executeTakeFirst();
    jsonOk(res, model);
};

export const getModels = async (req: AuthRequest, res: Response) => {
    const projectId = parseInt(req.params.project_id as string);
    // matching rust model `select_models` behavior via innerJoin
    const models = await db.selectFrom('Model')
        .selectAll()
        .where('project_id', '=', projectId)
        // Need to check project-user access if necessary, rust code queries `select_models` which presumably limits by project
        .limit(100)
        .offset(0)
        .execute();
    jsonOk(res, models);
};

// Use Multer for memory storage, then stream out to match Rust
// Rust uploads file named `{inserted_model_id}.glb` into `backend/assets/models/`
const UPLOADS_DIRECTORY = path.join(__dirname, '../../../backend/assets/models');

const storage = multer.diskStorage({
    destination: (req, file, cb) => {
        cb(null, UPLOADS_DIRECTORY);
    },
    filename: (req, file, cb) => {
        // Needs a model.id to be generated first. We can store temporarily or do in-memory and write after DB insert
        // OR better: handle DB insert, then return filename. Let's do it in the router.
        cb(null, 'temp_file'); // We'll move it down in the endpoint.
    }
});
export const upload = multer({ storage: multer.memoryStorage() }); // In memory for quick processing and rename

import fs from 'fs/promises';

export const uploadModel = async (req: AuthRequest, res: Response) => {
    const projectId = parseInt(req.params.project_id as string);
    const modelName = req.params.model_name as string as string;

    const insertedModel = await db.insertInto('Model')
        .values({
            version: 1,
            model_link: 'model_link', // from Rust
            name: req.params.model_name as string,
            project_id: projectId,
            created_by_id: req.user!.id,
            updated_at: new Date()
        })
        .returning('id')
        .executeTakeFirstOrThrow();

    if (req.file) {
        const targetPath = path.join(UPLOADS_DIRECTORY, `${insertedModel.id}.glb`);
        await fs.writeFile(targetPath, req.file.buffer);
    } // If multipart doesn't have file it continues

    const models = await db.selectFrom('Model')
        .selectAll()
        .where('project_id', '=', projectId)
        .limit(100)
        .offset(0)
        .execute();

    jsonOk(res, models);
};
