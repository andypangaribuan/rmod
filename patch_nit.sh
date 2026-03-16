cat << 'INNER_EOF' > patch_nit.diff
<<<<<<< SEARCH
            } else {
                #[allow(unused_variables)]
                let c = conn.detach();
            }
=======
            } else {
                let _ = conn.detach();
            }
>>>>>>> REPLACE
INNER_EOF
bash patch_nit.sh
