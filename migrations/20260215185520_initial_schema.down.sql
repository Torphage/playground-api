
-- ==========================================
-- DROP IN REVERSE ORDER OF CREATION
-- ==========================================

-- 6. Kitchen Schema
DROP TABLE IF EXISTS app_kitchen.culinary_item_tags;
DROP TABLE IF EXISTS app_kitchen.recipe_tags;
DROP TABLE IF EXISTS app_kitchen.recipe_notes;
DROP TABLE IF EXISTS app_kitchen.recipe_ingredients;
DROP TABLE IF EXISTS app_kitchen.recipes;
DROP TABLE IF EXISTS app_kitchen.ingredient_nutrients;
DROP TABLE IF EXISTS app_kitchen.ingredient_proposals;
DROP TABLE IF EXISTS app_kitchen.ingredient_conversions;
DROP TABLE IF EXISTS app_kitchen.ingredients;
DROP TABLE IF EXISTS app_kitchen.culinary_items;
DROP TABLE IF EXISTS app_kitchen.cuisines;
DROP TABLE IF EXISTS app_kitchen.tags;

-- 5. Shared Schema
DROP TABLE IF EXISTS shared.nutrients;
DROP TABLE IF EXISTS shared.data_sources;
DROP TABLE IF EXISTS shared.units;

-- 4. System Schema
DROP TABLE IF EXISTS system.feature_flags;
DROP TABLE IF EXISTS system.settings;

-- 3. Identity Schema
DROP TABLE IF EXISTS accounts.role_permissions;
DROP TABLE IF EXISTS accounts.permissions;
DROP TABLE IF EXISTS accounts.user_roles;
DROP TABLE IF EXISTS accounts.roles;
DROP TABLE IF EXISTS accounts.users;

-- 1. Schemas
DROP SCHEMA IF EXISTS app_kitchen;
DROP SCHEMA IF EXISTS shared;
DROP SCHEMA IF EXISTS system;
DROP SCHEMA IF EXISTS identity;
DROP SCHEMA IF EXISTS audit;