/*
  Warnings:

  - You are about to drop the column `shape` on the `Tag` table. All the data in the column will be lost.

*/
-- AlterTable
ALTER TABLE "Status" ADD COLUMN     "shape" "Shape" NOT NULL DEFAULT 'Cuboid';

-- AlterTable
ALTER TABLE "Tag" DROP COLUMN "shape";
