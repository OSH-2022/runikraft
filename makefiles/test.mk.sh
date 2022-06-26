#!/bin/sh

TEST_LIST='alloc_buddy0'

echo "# Generated by \`$0\` ." > $2

sed "s/@testlist@/$TEST_LIST/g" $1.0 >> $2
echo "" >> $2
for TEST in $TEST_LIST
do
	sed "s/@testname@/$TEST/g" $1.1 >> $2
	echo "" >> $2
done
