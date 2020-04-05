# v380-ipcam-firmware-patch

Patch the WiFi Smart Net Camera v380

1. Extract files from firmware patches
2. Write your own firmware patches

## Installation

Download binary from the [releases page](https://github.com/bcaller/v380-ipcam-firmware-patch/releases) or clone and build with `cargo`.

## Usage

### Read

```
$ patchv380 read AK3918E-V200_V.2.5.9.5/updatepatch/5d4315195544f84f54a52ac757ce200e.patch
Number of files: 21
exshell_afu.sh (1307 bytes, Script)
exshell_bfu.sh (606 bytes, Script)
IMG_KER_11 (2095680 bytes, Kernel Image)
IMG_MVS_mvs_v200_2595.sqsh4 (1024000 bytes, mtd3 Image)
isp_gc1034_quanjing.conf.gz (14875 bytes, Other)
isp_gc1034_yaotouji.conf.gz (14875 bytes, Other)
isp_sc1035_quanjing.conf.gz (21981 bytes, Other)
isp_sc1035_yaotouji.conf.gz (21981 bytes, Other)
isp_sc1135T.conf.gz (24938 bytes, Other)
isp_sc1135_quanjing.conf.gz (21970 bytes, Other)
isp_sc1135_yaotouji.conf.gz (14613 bytes, Other)
isp_sc1145_qiangji_180.conf.gz (13954 bytes, Other)
isp_sc1145_yaotouji.conf.gz (13943 bytes, Other)
isp_sc1235_quanjing.conf.gz (24308 bytes, Other)
isp_sc1235_yaotouji.conf.gz (15459 bytes, Other)
isp_sc1245_quanjing.conf.gz (15083 bytes, Other)
isp_sc1245_yaotouji.conf.gz (14565 bytes, Other)
prerun (53336 bytes, Other)
sf_stfailed_pwd_err_cn.wav (19896 bytes, Sound)
sf_stfailed_pwd_err_en.wav (22520 bytes, Sound)
stopallapp.sh (200 bytes, Other)
```

Add `-e ./unpackhere` to extract the files.

### Write

Make a firmware patch containing 2 files:

```
patchv380 write stuff/exshell_bfu.sh /path/to/IMG_RFS_stuff.sq4
```

You need to then move the `???.patch` file to `/sdcard/updatepatch/???.patch` and put in `/sdcard/localupdate.conf`:

```
[PATCH]
patchmd5=???
```

If you've already applied a patch, you might need `-h V380E2_CA` or whatever the ipcam's logs say you need.

## Hints

`exshell_bfu.sh` is run as root at the very start of the update process.
