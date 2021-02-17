
job=somejobid
git checkout $job

for (( i = 0; i < 500; i++ )); do
    git tag $i.0.0-a
done

git push --tags