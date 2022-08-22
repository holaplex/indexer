delete from auction_houses where address = any(array [
  'ah_address'
]);

delete from listings where id = any(array [
  '00000000-0000-0000-0001-000000000000'::uuid,
  '00000000-0000-0000-0001-000000000001',
  '00000000-0000-0000-0001-000000000002',
  '00000000-0000-0000-0001-000000000003'
]);

delete from purchases where id = any(array [
  '00000000-0000-0000-0002-000000000003'::uuid
]);
