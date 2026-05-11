-- Change status column from VARCHAR to ENUM
ALTER TABLE orders
MODIFY COLUMN status ENUM(
    'CART',
    'CHECKOUT',
    'PAYMENT_PENDING',
    'PAYMENT_FAILED',
    'CONFIRMED',
    'PROCESSING',
    'SHIPPED',
    'DELIVERED',
    'CANCELLED'
) NOT NULL;
