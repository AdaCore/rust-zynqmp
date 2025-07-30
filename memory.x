/* Sample memory map for the Xilinx ZynqMP ZCU102 board. */

MEMORY
{
  DDR (rwx) : ORIGIN = 0, LENGTH = 2048M
  OCM (rwx) : ORIGIN = 0xFFFC0000, LENGTH = 256K
}

REGION_ALIAS("CODE", DDR)
REGION_ALIAS("DATA", DDR)
