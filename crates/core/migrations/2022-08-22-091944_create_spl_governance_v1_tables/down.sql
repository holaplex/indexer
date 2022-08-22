drop table if exists vote_records_v1;
drop table if exists proposals_v1;
drop type if exists voteweightv1;

alter table signatory_records
rename to signatory_records_v2;

alter table token_owner_records
rename to token_owner_records_v2;