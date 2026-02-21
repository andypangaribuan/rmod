/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[macro_export]
macro_rules! vmove {
    ($($v:ident),+ , $blk:block) => {
        {
            $(let $v = $v.clone();)+
            async move $blk
        }
    };
}
