-- CreateEnum
CREATE TYPE "Shape" AS ENUM ('Cuboid', 'Tetrahedron', 'Capsule3d', 'Torus', 'Cylinder', 'Cone', 'ConicalFrustum', 'Sphere');

-- AlterTable
ALTER TABLE "Tag" ADD COLUMN     "rotation_x" REAL NOT NULL DEFAULT 0,
ADD COLUMN     "rotation_y" REAL NOT NULL DEFAULT 0,
ADD COLUMN     "rotation_z" REAL NOT NULL DEFAULT 0,
ADD COLUMN     "scale_x" REAL NOT NULL DEFAULT 0.3,
ADD COLUMN     "scale_y" REAL NOT NULL DEFAULT 0.3,
ADD COLUMN     "scale_z" REAL NOT NULL DEFAULT 0.3,
ADD COLUMN     "shape" "Shape" NOT NULL DEFAULT 'Cuboid';
