
for (( i = 0; i < 100; i++ )); do
    echo bla >> runs && git add --all && git commit -m "test" && git push origin master
done
