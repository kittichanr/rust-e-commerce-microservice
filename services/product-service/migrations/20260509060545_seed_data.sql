-- Add migration script here

USE product_db;

-- Electronics
INSERT INTO products (id, sku, name, description, price, stock_quantity, category, image_url, is_active) VALUES
('01934e5f-0001-7890-b123-000000000001', 'LAPTOP-001', 'Professional Laptop', 'High-performance laptop for developers and creators', 129999, 25, 'Electronics', 'https://example.com/laptop.jpg', true),
('01934e5f-0001-7890-b123-000000000002', 'PHONE-001', 'Smartphone Pro', 'Latest flagship smartphone with advanced camera', 99999, 50, 'Electronics', 'https://example.com/phone.jpg', true),
('01934e5f-0001-7890-b123-000000000003', 'TABLET-001', 'Tablet 10"', 'Lightweight tablet for reading and browsing', 49999, 30, 'Electronics', 'https://example.com/tablet.jpg', true),
('01934e5f-0001-7890-b123-000000000004', 'HEADPHONE-001', 'Wireless Headphones', 'Premium noise-cancelling wireless headphones', 29999, 100, 'Electronics', 'https://example.com/headphones.jpg', true);

-- Clothing
INSERT INTO products (id, sku, name, description, price, stock_quantity, category, image_url, is_active) VALUES
('01934e5f-0001-7890-b123-000000000005', 'TSHIRT-BLK-M', 'Classic Black T-Shirt (M)', 'Comfortable cotton t-shirt in black', 2999, 150, 'Clothing', 'https://example.com/tshirt-black.jpg', true),
('01934e5f-0001-7890-b123-000000000006', 'TSHIRT-WHT-M', 'Classic White T-Shirt (M)', 'Comfortable cotton t-shirt in white', 2999, 150, 'Clothing', 'https://example.com/tshirt-white.jpg', true),
('01934e5f-0001-7890-b123-000000000007', 'JEANS-001', 'Slim Fit Jeans', 'Modern slim fit denim jeans', 5999, 80, 'Clothing', 'https://example.com/jeans.jpg', true),
('01934e5f-0001-7890-b123-000000000008', 'JACKET-001', 'Winter Jacket', 'Warm winter jacket with hood', 8999, 40, 'Clothing', 'https://example.com/jacket.jpg', true);

-- Books
INSERT INTO products (id, sku, name, description, price, stock_quantity, category, image_url, is_active) VALUES
('01934e5f-0001-7890-b123-000000000009', 'BOOK-RUST-001', 'Rust Programming Book', 'Complete guide to Rust programming language', 3999, 75, 'Books', 'https://example.com/rust-book.jpg', true),
('01934e5f-0001-7890-b123-000000000010', 'BOOK-WEB-001', 'Web Development Guide', 'Modern web development with best practices', 4499, 60, 'Books', 'https://example.com/web-book.jpg', true),
('01934e5f-0001-7890-b123-000000000011', 'BOOK-DB-001', 'Database Design', 'Database design and optimization techniques', 3499, 50, 'Books', 'https://example.com/db-book.jpg', true);

-- Home & Kitchen
INSERT INTO products (id, sku, name, description, price, stock_quantity, category, image_url, is_active) VALUES
('01934e5f-0001-7890-b123-000000000012', 'COFFEE-001', 'Coffee Maker', 'Automatic drip coffee maker', 7999, 35, 'Home & Kitchen', 'https://example.com/coffee-maker.jpg', true),
('01934e5f-0001-7890-b123-000000000013', 'BLENDER-001', 'High-Speed Blender', 'Professional-grade blender for smoothies', 12999, 20, 'Home & Kitchen', 'https://example.com/blender.jpg', true),
('01934e5f-0001-7890-b123-000000000014', 'KETTLE-001', 'Electric Kettle', 'Fast-boiling electric kettle', 3999, 45, 'Home & Kitchen', 'https://example.com/kettle.jpg', true);

-- Sports & Outdoors
INSERT INTO products (id, sku, name, description, price, stock_quantity, category, image_url, is_active) VALUES
('01934e5f-0001-7890-b123-000000000015', 'YOGA-MAT-001', 'Yoga Mat', 'Non-slip yoga and exercise mat', 2999, 120, 'Sports & Outdoors', 'https://example.com/yoga-mat.jpg', true),
('01934e5f-0001-7890-b123-000000000016', 'DUMBBELL-001', 'Adjustable Dumbbells', 'Set of adjustable dumbbells 5-25kg', 15999, 25, 'Sports & Outdoors', 'https://example.com/dumbbells.jpg', true),
('01934e5f-0001-7890-b123-000000000017', 'BIKE-001', 'Mountain Bike', '21-speed mountain bike with suspension', 45999, 10, 'Sports & Outdoors', 'https://example.com/bike.jpg', true);

-- Some inactive/out-of-stock products
INSERT INTO products (id, sku, name, description, price, stock_quantity, category, image_url, is_active) VALUES
('01934e5f-0001-7890-b123-000000000018', 'CAMERA-OLD-001', 'Digital Camera (Discontinued)', 'Older model digital camera', 19999, 0, 'Electronics', 'https://example.com/camera-old.jpg', false),
('01934e5f-0001-7890-b123-000000000019', 'WATCH-001', 'Smartwatch', 'Fitness tracking smartwatch', 0, 0, 'Electronics', 'https://example.com/watch.jpg', false);

SELECT 'Sample data inserted successfully!' as status;
SELECT category, COUNT(*) as product_count FROM products GROUP BY category ORDER BY product_count DESC;
