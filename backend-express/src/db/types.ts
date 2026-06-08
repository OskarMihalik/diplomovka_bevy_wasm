import type { ColumnType } from "kysely";
export type Generated<T> = T extends ColumnType<infer S, infer I, infer U>
  ? ColumnType<S, I | undefined, U>
  : ColumnType<T, T | undefined, T>;
export type Timestamp = ColumnType<Date, Date | string, Date | string>;

export const Shape = {
    Cuboid: "Cuboid",
    Tetrahedron: "Tetrahedron",
    Capsule3d: "Capsule3d",
    Torus: "Torus",
    Cylinder: "Cylinder",
    Cone: "Cone",
    ConicalFrustum: "ConicalFrustum",
    Sphere: "Sphere"
} as const;
export type Shape = (typeof Shape)[keyof typeof Shape];
export type Model = {
    id: Generated<number>;
    version: number;
    model_link: string;
    name: string;
    created_at: Generated<Timestamp>;
    updated_at: Generated<Timestamp>;
    project_id: number;
    created_by_id: number;
};
export type Project = {
    id: Generated<number>;
    name: string;
    description: string | null;
    created_at: Generated<Timestamp>;
    updated_at: Generated<Timestamp>;
    created_by_id: number;
};
export type ProjectUser = {
    id: Generated<number>;
    project_id: number;
    is_admin: boolean;
    user_id: number;
    created_at: Generated<Timestamp>;
    updated_at: Generated<Timestamp>;
};
export type Status = {
    id: Generated<number>;
    title: string;
    created_at: Generated<Timestamp>;
    updated_at: Generated<Timestamp>;
    color_r: number;
    color_g: number;
    color_b: number;
    project_id: number;
    shape: Generated<Shape>;
};
export type Tag = {
    id: Generated<number>;
    title: string;
    created_at: Generated<Timestamp>;
    updated_at: Generated<Timestamp>;
    position_x: number;
    position_y: number;
    position_z: number;
    rotation_x: Generated<number>;
    rotation_y: Generated<number>;
    rotation_z: Generated<number>;
    scale_x: Generated<number>;
    scale_y: Generated<number>;
    scale_z: Generated<number>;
    project_id: number;
    created_by_id: number;
    status_id: number | null;
};
export type TagMessage = {
    id: Generated<number>;
    text: string;
    created_at: Generated<Timestamp>;
    updated_at: Generated<Timestamp>;
    tag_id: number;
    created_by_id: number;
};
export type User = {
    id: Generated<number>;
    email: string;
    username: string;
    password: string;
    salt: string;
    created_at: Generated<Timestamp>;
    updated_at: Generated<Timestamp>;
};
export type DB = {
    Model: Model;
    Project: Project;
    ProjectUser: ProjectUser;
    Status: Status;
    Tag: Tag;
    TagMessage: TagMessage;
    User: User;
};
