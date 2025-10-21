import { test, expect } from '@playwright/test';

test.describe('Admin Dashboard', () => {
  test('should load admin dashboard', async ({ page }) => {
    await page.goto('/admin');

    // Check if the main heading is visible
    await expect(page.getByRole('heading', { name: 'Admin Dashboard' })).toBeVisible();

    // Check if key metrics cards are present
    await expect(page.getByText('Active Sandboxes')).toBeVisible();
    await expect(page.getByText('Webhook Deliveries')).toBeVisible();
    await expect(page.getByText('DLQ Items')).toBeVisible();
    await expect(page.getByText('Monthly Revenue')).toBeVisible();
  });

  test('should navigate to DLQ management', async ({ page }) => {
    await page.goto('/admin/dlq');

    // Check if DLQ page loads
    await expect(page.getByRole('heading', { name: 'DLQ Management' })).toBeVisible();
  });

  test('should navigate to sandbox management', async ({ page }) => {
    await page.goto('/admin/sandbox');

    // Check if Sandbox page loads
    await expect(page.getByRole('heading', { name: 'Sandbox Management' })).toBeVisible();
    await expect(page.getByText('Create Sandbox')).toBeVisible();
  });

  test('should navigate to billing management', async ({ page }) => {
    await page.goto('/admin/billing');

    // Check if Billing page loads
    await expect(page.getByRole('heading', { name: 'Billing & Usage' })).toBeVisible();
    await expect(page.getByText('Total Revenue')).toBeVisible();
  });

  test('should navigate to webhooks management', async ({ page }) => {
    await page.goto('/admin/webhooks');

    // Check if Webhooks page loads
    await expect(page.getByRole('heading', { name: 'Webhooks Management' })).toBeVisible();
    await expect(page.getByText('Add Webhook')).toBeVisible();
  });

  test('should display sidebar navigation', async ({ page }) => {
    await page.goto('/admin');

    // Check if sidebar navigation items are visible (using more specific selectors)
    await expect(page.getByRole('button', { name: 'Dashboard' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'DLQ Management' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sandbox Management' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Billing & Usage' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Webhooks' })).toBeVisible();
  });
});