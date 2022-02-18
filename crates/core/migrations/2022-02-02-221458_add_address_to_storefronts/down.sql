alter table storefronts
drop column address,
add primary key (owner_address);