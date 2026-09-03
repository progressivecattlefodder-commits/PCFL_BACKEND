CREATE TABLE IF NOT EXISTS dynamic_qrs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(50) UNIQUE NOT NULL,
    target_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS qr_scan_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    qr_id UUID NOT NULL REFERENCES dynamic_qrs(id) ON DELETE CASCADE,
    ip_address VARCHAR(45),
    user_agent TEXT,
    device_type VARCHAR(20),
    os VARCHAR(50),
    browser VARCHAR(50),
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default master QR entry
INSERT INTO dynamic_qrs (slug, target_url)
VALUES ('main', 'https://progressivecattlefodderindustries.com')
ON CONFLICT (slug) DO NOTHING;